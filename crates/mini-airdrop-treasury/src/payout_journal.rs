//! Crash-safe payout reconciliation for a designated, already configured
//! settlement account. This module holds no signing key and creates no payment.
//! A caller supplies an approved, signed claim; retries dispatch identical bytes.
//! Finalized is established only from a QC-backed LedgerChain, never a send ACK.
use crate::{payout_message, TreasuryApprovedPayout};
use mini_airdrop::ClaimOutcome;
use mini_crypto::hash::blake3_256;
use mini_execution::LedgerChain;
use mini_settlement::{claim_digest, verify_claim_signature, CanonicalLedgerView, PaymentClaim};
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayoutPhase {
    Pending,
    Authorized,
    Submitted,
    Finalized,
}

#[cfg(test)]
mod tests {
    use super::*;
    use did_mini::{Capabilities, Controller, Did, Kel};
    use mini_chain::{
        sign_vote, BlockHeader, QuorumCertificate, ValidatorOracle, ValidatorSet, VoteKind,
    };
    use mini_crypto::SigningKey;
    use mini_execution::SettlementBlockBody;
    use mini_settlement::{sign_claim, MININET_NETWORK_ID};
    use mini_treasury::TreasurySignerSet;
    use std::collections::BTreeMap;

    struct Directory(BTreeMap<String, Kel>);
    impl ValidatorOracle for Directory {
        fn kel(&self, did: &Did) -> Option<&Kel> {
            self.0.get(did.scid())
        }
    }

    fn finalize(chain: &mut LedgerChain, claim: PaymentClaim) {
        let mut directory = Directory(BTreeMap::new());
        let signers: Vec<_> = (0..4)
            .map(|_| {
                let mut root = Controller::incept_single().unwrap();
                let device = Controller::incept_device_single_from_seeds(
                    &root.did(),
                    &mini_crypto::random::random_32().unwrap(),
                    &mini_crypto::random::random_32().unwrap(),
                )
                .unwrap();
                root.delegate_device(&device.did(), Capabilities::primary())
                    .unwrap();
                directory.0.insert(root.did().scid().into(), root.kel());
                directory.0.insert(device.did().scid().into(), device.kel());
                (root, device)
            })
            .collect();
        let validators = ValidatorSet::new(signers.iter().map(|(r, _)| r.did()).collect()).unwrap();
        let body = SettlementBlockBody::new(vec![claim]);
        let next = mini_execution::apply_block(chain.state(), &body).unwrap();
        let height = chain.height() + 1;
        let header = BlockHeader {
            height,
            prev_hash: chain.tip_hash(),
            state_root: next.commitment(),
            body_root: body.hash(),
            timestamp_ms: height,
            proposer: signers[0].0.did(),
        };
        let hash = header.hash();
        let qc = QuorumCertificate {
            height,
            round: 0,
            block_hash: hash,
            votes: signers[..3]
                .iter()
                .map(|(r, d)| sign_vote(VoteKind::Precommit, height, 0, hash, &r.did(), d))
                .collect(),
        };
        chain
            .apply_finalized_block(&header, &body, &qc, &validators, &directory)
            .unwrap();
    }

    fn approved(campaign: &[u8], outcome: &ClaimOutcome) -> TreasuryApprovedPayout {
        let signer = Controller::incept_single().unwrap();
        let signer_set = TreasurySignerSet::new(vec![signer.did()], 1).unwrap();
        let kel = signer.kel();
        let signatures = signer.sign_message(&payout_message(campaign, outcome));
        crate::verify_payout_approvals(campaign, outcome, &signer_set, &[(&kel, &signatures)])
            .unwrap()
    }
    fn fixture() -> (PathBuf, SigningKey, ClaimOutcome) {
        let root = std::env::temp_dir().join(format!(
            "payout-journal-{}-{}",
            std::process::id(),
            mini_crypto::random::random_32()
                .unwrap()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        ));
        let payer = SigningKey::generate().unwrap();
        let recipient = SigningKey::generate().unwrap();
        let outcome = ClaimOutcome {
            identity_root: Controller::incept_single().unwrap().did(),
            amount_micro: 100,
            recipient: recipient.verifying_key().to_bytes().to_vec(),
        };
        (root, payer, outcome)
    }

    #[test]
    fn failed_submission_reopens_and_finalizes_exact_payment_with_real_quorum() {
        let (root, payer, outcome) = fixture();
        let address = payer.verifying_key().to_bytes().to_vec();
        let approval = approved(b"campaign", &outcome);
        let claim = sign_claim(&payer, &outcome.recipient, 100, 0, 10_000, b"", 0).unwrap();
        let mut chain = LedgerChain::genesis_with_balances(
            100u64.into(),
            vec![(address.clone(), 100u64.into())],
        )
        .unwrap();
        let mut journal = PayoutJournal::open(
            &root,
            b"campaign",
            outcome.clone(),
            MININET_NETWORK_ID,
            address.clone(),
        )
        .unwrap();
        assert!(journal
            .submit(&approval, &claim, |_| panic!(
                "must not dispatch before authorization"
            ))
            .is_err());
        journal.authorize(&approval).unwrap();
        assert!(journal
            .submit(&approval, &claim, |_| Err(io::Error::other(
                "connection lost"
            )))
            .is_err());
        assert_eq!(journal.phase().unwrap(), PayoutPhase::Submitted);
        assert!(!journal.reconcile(&chain).unwrap());
        drop(journal);
        let mut journal =
            PayoutJournal::open(&root, b"campaign", outcome, MININET_NETWORK_ID, address).unwrap();
        assert_eq!(journal.pending_claim().unwrap(), Some(claim.clone()));
        journal
            .submit(&approval, &claim, |actual| {
                finalize(&mut chain, actual.clone());
                Ok(())
            })
            .unwrap();
        assert!(journal.reconcile(&chain).unwrap());
        assert_eq!(journal.phase().unwrap(), PayoutPhase::Finalized);
        journal
            .submit(&approval, &claim, |_| {
                panic!("must not re-dispatch finalized payout")
            })
            .unwrap();
        drop(journal);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cross_campaign_approval_and_changed_submitted_claim_are_refused() {
        let (root, payer, outcome) = fixture();
        let mut journal = PayoutJournal::open(
            &root,
            b"campaign",
            outcome.clone(),
            MININET_NETWORK_ID,
            payer.verifying_key().to_bytes().to_vec(),
        )
        .unwrap();
        assert!(journal.authorize(&approved(b"other", &outcome)).is_err());
        let approval = approved(b"campaign", &outcome);
        journal.authorize(&approval).unwrap();
        let first = sign_claim(&payer, &outcome.recipient, 100, 0, 10_000, b"", 0).unwrap();
        journal.submit(&approval, &first, |_| Ok(())).unwrap();
        let second = sign_claim(&payer, &outcome.recipient, 100, 1, 10_000, b"", 0).unwrap();
        assert!(journal
            .submit(&approval, &second, |_| panic!(
                "second payment must not escape"
            ))
            .is_err());
        assert_eq!(journal.pending_claim().unwrap(), Some(first));
        drop(journal);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[derive(Debug)]
pub struct PayoutJournal {
    path: PathBuf,
    binding: Vec<u8>,
    campaign: Vec<u8>,
    outcome: ClaimOutcome,
    network: [u8; 32],
    payer: Vec<u8>,
    _lock: File,
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

impl PayoutJournal {
    /// The configured payer/network are local trust inputs, not inferred from
    /// an untrusted payment. Campaign+identity select one stable journal forever.
    pub fn open(
        directory: &Path,
        campaign: &[u8],
        outcome: ClaimOutcome,
        network: [u8; 32],
        payer: Vec<u8>,
    ) -> io::Result<Self> {
        if campaign.len() > mini_airdrop::MAX_CAMPAIGN_ID_BYTES
            || payer.len() != 32
            || outcome.recipient.len() > mini_airdrop::MAX_RECIPIENT_BYTES
            || outcome.amount_micro == 0
        {
            return Err(invalid("invalid payout configuration"));
        }
        mini_durable::create_dir_all(directory)?;
        let mut key = b"mini-airdrop-treasury/payout-key/v1".to_vec();
        key.extend_from_slice(&(campaign.len() as u32).to_be_bytes());
        key.extend_from_slice(campaign);
        key.extend_from_slice(outcome.identity_root.as_str().as_bytes());
        let name: String = blake3_256(&key)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        let lock = mini_durable::try_lock_exclusive(&directory.join(format!("{name}.lock")))?;
        let path = directory.join(format!("{name}.payout"));
        let mut binding = b"mini-airdrop-treasury/payout-journal/v1".to_vec();
        binding.extend_from_slice(&network);
        binding.extend_from_slice(&payer);
        binding.extend_from_slice(&payout_message(campaign, &outcome));
        let journal = Self {
            path,
            binding,
            campaign: campaign.to_vec(),
            outcome,
            network,
            payer,
            _lock: lock,
        };
        if !journal.path.try_exists()? {
            journal.write(PayoutPhase::Pending, None)?;
        }
        journal.read()?;
        Ok(journal)
    }

    /// Locally recorded progress, not independently authenticated finality.
    pub fn phase(&self) -> io::Result<PayoutPhase> {
        Ok(self.read()?.0)
    }
    pub fn pending_claim(&self) -> io::Result<Option<PaymentClaim>> {
        Ok(self.read()?.1)
    }

    pub fn authorize(&mut self, approval: &TreasuryApprovedPayout) -> io::Result<()> {
        self.check_approval(approval)?;
        let (phase, claim) = self.read()?;
        if phase == PayoutPhase::Pending {
            self.write(PayoutPhase::Authorized, None)?;
        } else {
            self.write(phase, claim.as_ref())?;
        }
        Ok(())
    }

    /// Persist Submitted *before* dispatch. Submission can be attempted more
    /// than once after an ambiguous failure, but always with the exact same
    /// signed claim; settlement's payer+sequence+digest enforces idempotence.
    pub fn submit(
        &mut self,
        approval: &TreasuryApprovedPayout,
        claim: &PaymentClaim,
        dispatch: impl FnOnce(&PaymentClaim) -> io::Result<()>,
    ) -> io::Result<()> {
        self.check_approval(approval)?;
        self.check_claim(claim)?;
        let (phase, prior) = self.read()?;
        if phase == PayoutPhase::Pending {
            return Err(invalid("payout is not authorized"));
        }
        if let Some(prior) = &prior {
            if prior != claim {
                return Err(invalid("cannot replace an already submitted payment"));
            }
        }
        if phase == PayoutPhase::Finalized {
            return Ok(());
        }
        self.write(PayoutPhase::Submitted, Some(claim))?;
        dispatch(claim)
    }

    /// Returns true only when the supplied canonical chain proves this exact
    /// claim executed. A conflicting payer sequence or a rejection fails closed.
    pub fn reconcile(&mut self, chain: &LedgerChain) -> io::Result<bool> {
        if chain.state().network_id() != self.network {
            return Err(invalid("wrong settlement network"));
        }
        let (_, claim) = self.read()?;
        let Some(claim) = claim else {
            return Ok(false);
        };
        let digest = claim_digest(&claim);
        match chain
            .state()
            .finalized_claim_digest(&claim.payer, claim.sequence)
        {
            Some(actual) if actual == digest => {
                self.write(PayoutPhase::Finalized, Some(&claim))?;
                Ok(true)
            }
            Some(_) => Err(invalid(
                "canonical sequence was consumed by another payment",
            )),
            None if chain.state().rejected_claim(&digest).is_some() => Err(invalid(
                "canonical payment rejection; manual reconciliation required",
            )),
            None => Ok(false),
        }
    }

    fn check_approval(&self, approval: &TreasuryApprovedPayout) -> io::Result<()> {
        if approval.campaign_id() != self.campaign || approval.outcome() != &self.outcome {
            return Err(invalid("approval does not bind this campaign and payout"));
        }
        Ok(())
    }
    fn check_claim(&self, claim: &PaymentClaim) -> io::Result<()> {
        if claim.network_id != self.network
            || claim.payer != self.payer
            || claim.payee != self.outcome.recipient
            || claim.amount_micro != self.outcome.amount_micro
            || verify_claim_signature(claim).is_err()
        {
            return Err(invalid(
                "payment does not match authorized payout/account/network",
            ));
        }
        Ok(())
    }
    fn write(&self, phase: PayoutPhase, claim: Option<&PaymentClaim>) -> io::Result<()> {
        let mut bytes = self.binding.clone();
        bytes.push(match phase {
            PayoutPhase::Pending => 0,
            PayoutPhase::Authorized => 1,
            PayoutPhase::Submitted => 2,
            PayoutPhase::Finalized => 3,
        });
        let wire = claim
            .map(PaymentClaim::to_wire_bytes)
            .transpose()
            .map_err(|e| invalid(&e.to_string()))?
            .unwrap_or_default();
        bytes.extend_from_slice(&(wire.len() as u32).to_be_bytes());
        bytes.extend(wire);
        let checksum = blake3_256(&bytes);
        bytes.extend_from_slice(&checksum);
        mini_durable::atomic_replace(&self.path, &bytes)
    }
    fn read(&self) -> io::Result<(PayoutPhase, Option<PaymentClaim>)> {
        let mut bytes = Vec::new();
        File::open(&self.path)?
            .take(32769)
            .read_to_end(&mut bytes)?;
        let start = self.binding.len();
        if bytes.len() > 32768 || bytes.len() < start + 37 || !bytes.starts_with(&self.binding) {
            return Err(invalid("corrupt or transplanted payout journal"));
        }
        let phase = match bytes[start] {
            0 => PayoutPhase::Pending,
            1 => PayoutPhase::Authorized,
            2 => PayoutPhase::Submitted,
            3 => PayoutPhase::Finalized,
            _ => return Err(invalid("unknown payout phase")),
        };
        let len = u32::from_be_bytes(bytes[start + 1..start + 5].try_into().unwrap()) as usize;
        if len > 16384
            || bytes.len() != start + 5 + len + 32
            || blake3_256(&bytes[..bytes.len() - 32]) != bytes[bytes.len() - 32..]
        {
            return Err(invalid("corrupt payout record"));
        }
        let requires_claim = matches!(phase, PayoutPhase::Submitted | PayoutPhase::Finalized);
        if requires_claim != (len != 0) {
            return Err(invalid("inconsistent payout phase"));
        }
        let claim = if len == 0 {
            None
        } else {
            let claim = PaymentClaim::from_wire_bytes(&bytes[start + 5..start + 5 + len])
                .map_err(|e| invalid(&e.to_string()))?;
            self.check_claim(&claim)?;
            Some(claim)
        };
        Ok((phase, claim))
    }
}
