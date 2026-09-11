//! Real proof -> canonical membership -> honest vote -> QC -> output re-spend.
use std::{collections::BTreeMap, sync::Arc};

use did_mini::{Capabilities, Controller, Did, Kel};
use mini_chain::{
    sign_vote, BlockHeader, QuorumCertificate, ValidatorOracle, ValidatorSet, VoteKind,
};
use mini_consensus::{
    proposer_for, sign_proposal, ConsensusMessage, ConsensusNode, ConsensusSnapshot, Emit,
    FinalizedBlock, NodeConfig, StateSyncPayload, StateSyncResponse, NIL,
};
use mini_execution::{
    apply_block_with_verifier, ExecutionError, LedgerChain, NullifierRecord, SettlementBlockBody,
    ShieldedGenesisAllocation, ShieldedOutput,
};
use mini_private_payment::{
    build, scan_one, verify, InMemoryOutputSet, OutputSet, PaymentPurpose, PaymentRequest,
    PrivatePaymentClaim, Recipient, SpendableOutput, MIN_RING_SIZE,
};
use mini_shielded_verify::{ClaimEvidencePool, ShieldedClaimVerifier};
use mini_value::{derive_spend_scalar, public_amount_commitment, StealthKeypair};

const NETWORK: [u8; 32] = [0x59; 32];

#[derive(Clone, Default)]
struct Directory(BTreeMap<String, Kel>);
impl ValidatorOracle for Directory {
    fn kel(&self, did: &Did) -> Option<&Kel> {
        self.0.get(did.scid())
    }
}

struct Fixture {
    owners: Vec<StealthKeypair>,
    outputs: InMemoryOutputSet,
    genesis: LedgerChain,
    evidence: Arc<ClaimEvidencePool>,
    verifier: Arc<ShieldedClaimVerifier>,
    signers: Vec<(Controller, Controller)>,
    validators: ValidatorSet,
    directory: Directory,
}
impl Fixture {
    fn new() -> Self {
        let evidence = Arc::new(ClaimEvidencePool::new());
        let verifier = Arc::new(ShieldedClaimVerifier::new(NETWORK, evidence.clone()));
        let owners: Vec<_> = (0..MIN_RING_SIZE)
            .map(|_| StealthKeypair::generate().unwrap())
            .collect();
        let mut allocations = Vec::new();
        let mut outputs = InMemoryOutputSet::new();
        for owner in &owners {
            let key = owner.spend_public_bytes().to_vec();
            let commitment = public_amount_commitment(1_000).to_vec();
            outputs.push(key.clone(), commitment.clone());
            allocations.push(ShieldedGenesisAllocation {
                output: ShieldedOutput {
                    public_key: key,
                    amount_commitment: commitment,
                },
                amount_micro: 1_000,
            });
        }
        let genesis =
            LedgerChain::genesis_with_shielded_allocations(NETWORK, allocations, verifier.as_ref())
                .unwrap();
        let mut directory = Directory::default();
        let signers: Vec<_> = (0..4u8)
            .map(|index| {
                let seed = 20 + index * 10;
                let mut root =
                    Controller::incept_single_from_seeds(&[seed; 32], &[seed + 1; 32]).unwrap();
                let device = Controller::incept_device_single_from_seeds(
                    &root.did(),
                    &[seed + 2; 32],
                    &[seed + 3; 32],
                )
                .unwrap();
                root.delegate_device(&device.did(), Capabilities::primary())
                    .unwrap();
                directory.0.insert(root.did().scid().to_owned(), root.kel());
                directory
                    .0
                    .insert(device.did().scid().to_owned(), device.kel());
                (root, device)
            })
            .collect();
        let validators =
            ValidatorSet::new(signers.iter().map(|(root, _)| root.did()).collect()).unwrap();
        Self {
            owners,
            outputs,
            genesis,
            evidence,
            verifier,
            signers,
            validators,
            directory,
        }
    }

    fn node(&self, index: usize) -> ConsensusNode<Directory> {
        let seed = 20 + index as u8 * 10;
        let root = self.signers[index].0.did();
        let device =
            Controller::incept_device_single_from_seeds(&root, &[seed + 2; 32], &[seed + 3; 32])
                .unwrap();
        ConsensusNode::new_with_genesis(
            NodeConfig {
                root,
                device,
                validators: self.validators.clone(),
                oracle: self.directory.clone(),
                body_source: Box::new(|_| SettlementBlockBody::default()),
            },
            self.genesis.clone(),
            Some(self.verifier.clone()),
        )
        .unwrap()
    }

    fn seal(
        &self,
        chain: &LedgerChain,
        body: SettlementBlockBody,
        state_root: [u8; 32],
    ) -> FinalizedBlock {
        let height = chain.height() + 1;
        let header = BlockHeader {
            height,
            prev_hash: chain.tip_hash(),
            body_root: body.hash(),
            state_root,
            timestamp_ms: height,
            proposer: proposer_for(height, 0, &self.validators).clone(),
        };
        let hash = header.hash();
        let votes = self.signers[..3]
            .iter()
            .map(|(root, device)| {
                sign_vote(VoteKind::Precommit, height, 0, hash, &root.did(), device)
            })
            .collect();
        FinalizedBlock {
            header,
            body,
            qc: QuorumCertificate {
                height,
                round: 0,
                block_hash: hash,
                votes,
            },
        }
    }

    fn finalized(&self, chain: &mut LedgerChain, claim: &PrivatePaymentClaim) -> FinalizedBlock {
        self.evidence.insert(claim.encode()).unwrap();
        let body = body(claim);
        let next =
            apply_block_with_verifier(chain.state(), &body, Some(self.verifier.as_ref())).unwrap();
        let block = self.seal(chain, body, next.commitment());
        chain
            .apply_finalized_block_with_verifier(
                &block.header,
                &block.body,
                &block.qc,
                &self.validators,
                &self.directory,
                Some(self.verifier.as_ref()),
            )
            .unwrap();
        block
    }

    fn rejects_at_vote_and_recovery(
        &self,
        claim: &PrivatePaymentClaim,
        body: SettlementBlockBody,
        expected: ExecutionError,
    ) {
        self.evidence.insert(claim.encode()).unwrap();
        assert_eq!(
            apply_block_with_verifier(self.genesis.state(), &body, Some(self.verifier.as_ref())),
            Err(expected.clone())
        );
        let block = self.seal(&self.genesis, body, self.genesis.state().commitment());
        let proposer = self
            .signers
            .iter()
            .position(|(root, _)| root.did() == block.header.proposer)
            .unwrap();
        let receiver = (proposer + 1) % self.signers.len();
        let mut node = self.node(receiver);
        node.start().unwrap();
        let (root, device) = &self.signers[proposer];
        let proposal = sign_proposal(
            0,
            -1,
            block.header.clone(),
            block.body.clone(),
            &root.did(),
            device,
        );
        let emits = node
            .on_message(ConsensusMessage::Proposal(proposal))
            .unwrap();
        assert!(emits.iter().any(|emit| matches!(emit, Emit::Broadcast(ConsensusMessage::Vote(vote)) if vote.kind == VoteKind::Prevote && vote.block_hash == NIL)));
        assert_eq!(
            node.catch_up(vec![block]).unwrap_err(),
            mini_consensus::ConsensusError::Execution(expected)
        );
        assert_eq!(node.finalized_height(), 0);
        assert_eq!(node.state(), self.genesis.state());
    }
}

fn body(claim: &PrivatePaymentClaim) -> SettlementBlockBody {
    let digest = claim.transcript_digest();
    SettlementBlockBody::default().with_nullifiers(
        claim
            .inputs
            .iter()
            .map(|input| NullifierRecord::new(input.signature.key_image.clone(), digest))
            .collect(),
    )
}

fn payment(
    outputs: &InMemoryOutputSet,
    spends: Vec<SpendableOutput>,
    recipient: &StealthKeypair,
    amount: u64,
    fee: u64,
) -> PrivatePaymentClaim {
    build(
        &PaymentRequest {
            network_id: NETWORK,
            spends,
            recipients: vec![Recipient {
                spend_public: recipient.spend_public_bytes().to_vec(),
                view_public: recipient.view_public_bytes().to_vec(),
                amount_micro: amount,
                purpose: PaymentPurpose::new(b"canonical-test".to_vec()),
            }],
            fee_micro: fee,
            ring_size: MIN_RING_SIZE,
            valid_until_ms: 10_000,
            last_known_chain: vec![],
            decoy_entropy: mini_crypto::random_32().unwrap(),
        },
        outputs,
    )
    .unwrap()
    .0
}

#[test]
fn a_real_qc_finalizes_a_payment_and_its_recipient_spends_the_new_output() {
    let mut fixture = Fixture::new();
    let recipient = StealthKeypair::generate().unwrap();
    let first = payment(
        &fixture.outputs,
        vec![SpendableOutput {
            set_index: 0,
            one_time_secret: fixture.owners[0].spend_secret_bytes(),
            value_micro: 1_000,
            blinding: [0; 32],
        }],
        &recipient,
        990,
        10,
    );
    let mut chain = fixture.genesis.clone();
    let first_block = fixture.finalized(&mut chain, &first);
    assert_eq!(chain.height(), 1);
    let verified = verify(&first, &NETWORK).unwrap();
    let received = scan_one(
        &recipient.view_secret_bytes(),
        &recipient.spend_public_bytes(),
        &verified,
    )
    .unwrap();
    let note = &received[0].note;
    let output = &first.outputs[received[0].output_index];
    assert_eq!(
        chain
            .state()
            .shielded_outputs()
            .get(&output.output.one_time_address),
        Some(&output.amount_commitment)
    );
    let scalar = derive_spend_scalar(
        &recipient.view_secret_bytes(),
        &recipient.spend_secret_bytes(),
        &output.output,
    )
    .unwrap();
    fixture.outputs.push(
        output.output.one_time_address.clone(),
        output.amount_commitment.clone(),
    );
    let second = payment(
        &fixture.outputs,
        vec![SpendableOutput {
            set_index: fixture.outputs.len() - 1,
            one_time_secret: scalar.to_bytes(),
            value_micro: note.amount_micro,
            blinding: note.blinding,
        }],
        &fixture.owners[1],
        985,
        5,
    );
    let second_block = fixture.finalized(&mut chain, &second);
    assert_eq!(chain.state().nullifier_count(), 2);
    assert_eq!(
        chain.state().shielded_pool().as_micro(),
        (MIN_RING_SIZE as u128) * 1_000 - 15
    );
    assert_eq!(chain.state().unallocated_circulating().as_micro(), 15);
    chain.state().verify_supply_conservation().unwrap();

    let mut node = fixture.node(0);
    node.catch_up(vec![first_block, second_block.clone()])
        .unwrap();
    assert_eq!(node.state(), chain.state());
    let snapshot = ConsensusSnapshot::new(
        second_block.header.clone(),
        second_block.qc.clone(),
        chain.state().clone(),
    )
    .unwrap();
    let restored = snapshot
        .clone()
        .into_chain_with_verifier(
            NETWORK,
            &fixture.validators,
            &fixture.directory,
            fixture.genesis.state().shielded_genesis_commitment(),
            Some(fixture.verifier.as_ref()),
        )
        .unwrap();
    assert_eq!(restored.state(), chain.state());
    assert!(snapshot
        .clone()
        .into_chain_with_verifier(
            NETWORK,
            &fixture.validators,
            &fixture.directory,
            fixture.genesis.state().shielded_genesis_commitment(),
            None
        )
        .is_err());
    let mut snapshot_node = fixture.node(0);
    snapshot_node
        .apply_state_sync(StateSyncResponse {
            network_id: NETWORK,
            payload: StateSyncPayload::Snapshot {
                snapshot: Box::new(snapshot),
                blocks: vec![],
            },
        })
        .unwrap();
    assert_eq!(snapshot_node.state(), chain.state());
}

#[test]
fn genuine_proofs_over_invented_inputs_never_receive_an_honest_vote_or_enter_recovery() {
    let fixture = Fixture::new();
    let invented = Fixture::new();
    let claim = payment(
        &invented.outputs,
        vec![SpendableOutput {
            set_index: 0,
            one_time_secret: invented.owners[0].spend_secret_bytes(),
            value_micro: 1_000,
            blinding: [0; 32],
        }],
        &fixture.owners[1],
        990,
        10,
    );
    assert!(verify(&claim, &NETWORK).is_ok());
    fixture.rejects_at_vote_and_recovery(
        &claim,
        body(&claim),
        ExecutionError::UnknownShieldedInput,
    );
}

#[test]
fn a_valid_proof_cannot_rebind_a_canonical_key_to_an_inflated_commitment() {
    let fixture = Fixture::new();
    let mut rebound = InMemoryOutputSet::new();
    for (index, owner) in fixture.owners.iter().enumerate() {
        rebound.push(
            owner.spend_public_bytes().to_vec(),
            public_amount_commitment(if index == 0 { 2_000 } else { 1_000 }),
        );
    }
    let claim = payment(
        &rebound,
        vec![SpendableOutput {
            set_index: 0,
            one_time_secret: fixture.owners[0].spend_secret_bytes(),
            value_micro: 2_000,
            blinding: [0; 32],
        }],
        &fixture.owners[1],
        1_990,
        10,
    );
    assert!(verify(&claim, &NETWORK).is_ok());
    fixture.rejects_at_vote_and_recovery(
        &claim,
        body(&claim),
        ExecutionError::UnknownShieldedInput,
    );
}

#[test]
fn copied_key_images_substituted_digests_and_partial_groups_never_enter_qc_state() {
    let fixture = Fixture::new();
    let claim = payment(
        &fixture.outputs,
        (0..2)
            .map(|index| SpendableOutput {
                set_index: index,
                one_time_secret: fixture.owners[index].spend_secret_bytes(),
                value_micro: 1_000,
                blinding: [0; 32],
            })
            .collect(),
        &fixture.owners[2],
        1_990,
        10,
    );
    let complete = body(&claim);
    let mut copied = complete.clone();
    copied.nullifiers[0].key_image = vec![0x42; 32];
    let mut substituted = complete.clone();
    for record in &mut substituted.nullifiers {
        record.claim_digest = [0x44; 32];
    }
    let mut partial = complete;
    partial.nullifiers.pop();
    for attack in [copied, substituted, partial] {
        fixture.rejects_at_vote_and_recovery(&claim, attack, ExecutionError::InvalidShieldedClaim);
    }
}
