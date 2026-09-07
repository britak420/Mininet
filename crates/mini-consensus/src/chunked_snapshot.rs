//! Chunked, Merkle-authenticated execution-state transfer (roadmap #45,
//! D-0207's own "Required follow-up": "chunked authenticated state
//! transfer"; Directive 11, weak-device operation).
//!
//! [`ConsensusSnapshot::to_wire_bytes`](crate::snapshot::ConsensusSnapshot::to_wire_bytes)
//! caps a full execution-state snapshot to one ~16 MiB encrypted bearer
//! frame, by its own doc comment's design: "a snapshot too large for one
//! encrypted bearer frame is rejected; chunked/Merkle state transfer is a
//! separately scoped future format, never an implicit unbounded fallback."
//! This module is that separately scoped format.
//!
//! ## What chunking buys, and what it does not
//!
//! [`SnapshotManifest`] carries the same [`mini_chain::BlockHeader`]/
//! [`mini_chain::QuorumCertificate`] finality binding an ordinary
//! [`ConsensusSnapshot`] does, plus a Merkle root (`chunks_root`) over the
//! encoded execution state split into fixed-size byte chunks. A receiver
//! still trusts exactly one thing at the end —
//! `header.state_root == state.commitment()`, unchanged, checked once by
//! [`SnapshotAssembler::finish`] the same way `ConsensusSnapshot::
//! from_wire_bytes` already does for the single-frame case. `chunks_root`
//! adds nothing to that trust boundary: a dishonest peer can hand out a
//! perfectly self-consistent root over garbage bytes just as it could hand
//! out a dishonest un-chunked snapshot today, and the final commitment
//! check catches both identically. What `chunks_root` buys is entirely
//! operational: a receiver on a slow or lossy link can verify each chunk
//! *as it arrives* and re-request only the one bad chunk, instead of
//! discovering a single flipped byte only after downloading the entire
//! multi-megabyte state and decoding it — and, once a caller sources chunks
//! from more than one peer (not built here), every peer's chunks are
//! checked against the same manifest root before being trusted at all.
//!
//! ## Why a local Merkle tree instead of a `mini-spacetime` dependency
//!
//! `mini_spacetime::merkle::MerkleTree`/`MerkleProof` already implement the
//! identical RFC 6962-style leaf/node domain separation over BLAKE3 — but
//! that crate is proof-of-space-time-specific (`mini-porep`,
//! `mini-storage-fraud` are its only consumers today), its `MerkleProof`
//! has no wire codec yet ("proofs travel only in-process today", by its own
//! doc comment), and depending on it here would wire this crate's chain
//! transport layer to an unrelated storage-proof crate for the sake of
//! avoiding roughly eighty lines of a standard, already-reviewed
//! construction. This module reimplements that same construction locally,
//! scoped to what a wire-transportable chunk proof actually needs —
//! composition of prior art already used in this tree, not new
//! cryptography (project convention).
//!
//! ## Scope: manifest, chunking, and verified reassembly only
//!
//! No network wiring: [`crate::net`]'s `catch_up_over_tcp`/
//! `state_sync_over_tcp` are untouched, and no [`ChunkRequest`]/
//! [`ChunkResponse`] ever crosses a real socket here — that is a follow-up
//! slice, mirroring how `did_mini::witness_protocol` (D-0464) shipped pure
//! request/response types before `mini_sync::gossip` (D-0467) wired an
//! unrelated protocol onto real transport. Also not built: multi-peer chunk
//! sourcing, retry/backoff policy, and eclipse-resistant peer selection —
//! all still named in D-0207's own "Required follow-up".

use mini_chain::{BlockHeader, QuorumCertificate, ValidatorOracle, ValidatorSet};
use mini_crypto::HashAlgorithm;
use mini_execution::{LedgerState, MAX_LEDGER_SNAPSHOT_BYTES};

use crate::catchup::{decode_qc, encode_qc};
use crate::error::{ConsensusError, Result};
use crate::snapshot::ConsensusSnapshot;
use crate::wire::{decode_header, encode_header, put_bytes, Reader};

const DOMAIN: &[u8] = b"mini-consensus/chunked-snapshot/v1";
const LEAF_PREFIX: u8 = 0x00;
const NODE_PREFIX: u8 = 0x01;

/// Smallest chunk a caller may choose. Far above zero so a manifest can
/// never declare an unbounded chunk count for a bounded
/// [`MAX_LEDGER_SNAPSHOT_BYTES`] state.
pub const MIN_CHUNK_BYTES: usize = 1024;

/// Largest chunk a caller may choose. Above this, chunking buys nothing
/// over today's single-frame transfer.
pub const MAX_CHUNK_BYTES: usize = 1024 * 1024;

/// Hard cap on chunks in one manifest: the smallest allowed chunk size
/// against the largest possible snapshot.
pub const MAX_CHUNKS: usize = MAX_LEDGER_SNAPSHOT_BYTES / MIN_CHUNK_BYTES;

fn leaf_hash(chunk: &[u8]) -> [u8; 32] {
    let mut buf = Vec::with_capacity(1 + chunk.len());
    buf.push(LEAF_PREFIX);
    buf.extend_from_slice(chunk);
    HashAlgorithm::Blake3.digest(&buf)
}

fn node_hash(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut buf = [0u8; 65];
    buf[0] = NODE_PREFIX;
    buf[1..33].copy_from_slice(left);
    buf[33..65].copy_from_slice(right);
    HashAlgorithm::Blake3.digest(&buf)
}

/// Every level of a chunk Merkle tree, leaves first, root last. An unpaired
/// final node at any level is promoted unchanged rather than duplicated —
/// the same odd-node rule `mini_spacetime::merkle::MerkleTree` uses, so an
/// unbalanced chunk count never has an ambiguous or attacker-chosen pairing.
fn build_levels(leaves: Vec<[u8; 32]>) -> Vec<Vec<[u8; 32]>> {
    let mut levels = vec![leaves];
    while levels.last().expect("levels is never empty").len() > 1 {
        let current = levels.last().expect("levels is never empty");
        let mut next = Vec::with_capacity(current.len().div_ceil(2));
        let mut i = 0;
        while i < current.len() {
            if i + 1 < current.len() {
                next.push(node_hash(&current[i], &current[i + 1]));
            } else {
                next.push(current[i]);
            }
            i += 2;
        }
        levels.push(next);
    }
    levels
}

/// How many sibling levels a proof for `leaf_count` leaves must carry,
/// mirroring [`build_levels`]'s repeated halving exactly.
fn proof_depth(leaf_count: usize) -> usize {
    let mut width = leaf_count;
    let mut depth = 0;
    while width > 1 {
        width = width.div_ceil(2);
        depth += 1;
    }
    depth
}

fn prove(levels: &[Vec<[u8; 32]>], index: usize) -> ChunkProof {
    let mut siblings = Vec::with_capacity(levels.len().saturating_sub(1));
    let mut idx = index;
    for level in &levels[..levels.len() - 1] {
        let is_right = idx % 2 == 1;
        let sibling_idx = if is_right { idx - 1 } else { idx + 1 };
        siblings.push(level.get(sibling_idx).copied());
        idx /= 2;
    }
    ChunkProof { siblings }
}

/// A membership proof that one chunk belongs at one index of a manifest's
/// `chunks_root`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkProof {
    siblings: Vec<Option<[u8; 32]>>,
}

impl ChunkProof {
    /// Verify `chunk` belongs at `index` of `chunk_count` total chunks under
    /// `root`. The sibling list must have exactly the length `chunk_count`
    /// implies — a mismatched length is rejected outright, mirroring
    /// `mini_spacetime::merkle::MerkleProof::verify`'s own hardening against
    /// a proof with more than one valid encoding.
    pub fn verify(&self, chunk: &[u8], index: usize, chunk_count: usize, root: [u8; 32]) -> bool {
        if chunk_count == 0 || index >= chunk_count {
            return false;
        }
        if self.siblings.len() != proof_depth(chunk_count) {
            return false;
        }
        let mut hash = leaf_hash(chunk);
        let mut idx = index;
        for sibling in &self.siblings {
            hash = match sibling {
                Some(sib) => {
                    if idx % 2 == 1 {
                        node_hash(sib, &hash)
                    } else {
                        node_hash(&hash, sib)
                    }
                }
                None => hash,
            };
            idx /= 2;
        }
        hash == root
    }

    fn to_wire_bytes(&self) -> Vec<u8> {
        let mut w = Vec::new();
        w.extend_from_slice(&(self.siblings.len() as u32).to_be_bytes());
        for sibling in &self.siblings {
            match sibling {
                Some(hash) => {
                    w.push(1);
                    w.extend_from_slice(hash);
                }
                None => w.push(0),
            }
        }
        w
    }

    fn from_wire_bytes(r: &mut Reader<'_>) -> Result<Self> {
        let count = r.u32()? as usize;
        // A proof's depth is at most log2(MAX_CHUNKS); this bound only
        // stops a lying count from forcing a large speculative allocation
        // before the (much smaller) real bytes are read.
        if count > MAX_CHUNKS {
            return Err(ConsensusError::TooLarge);
        }
        let mut siblings = Vec::with_capacity(count.min(64));
        for _ in 0..count {
            siblings.push(match r.u8()? {
                0 => None,
                1 => {
                    let mut hash = [0u8; 32];
                    hash.copy_from_slice(r.take(32)?);
                    Some(hash)
                }
                _ => return Err(ConsensusError::Malformed),
            });
        }
        Ok(ChunkProof { siblings })
    }
}

/// A chunk-authenticated description of one finalized execution-state
/// snapshot: the same finality binding as [`ConsensusSnapshot`], plus a
/// Merkle root over the state bytes split into fixed-size chunks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotManifest {
    pub header: BlockHeader,
    pub qc: QuorumCertificate,
    /// Exact byte length of the encoded execution state, before chunking.
    pub total_len: u64,
    /// Byte size of every chunk except possibly the last.
    pub chunk_size: u32,
    /// `ceil(total_len / chunk_size)`.
    pub chunk_count: u32,
    /// Merkle root over `chunk_count` leaf hashes, one per chunk, in order.
    pub chunks_root: [u8; 32],
}

impl SnapshotManifest {
    pub fn to_wire_bytes(&self) -> Result<Vec<u8>> {
        let mut w = Vec::new();
        w.extend_from_slice(DOMAIN);
        encode_header(&mut w, &self.header);
        encode_qc(&mut w, &self.qc);
        w.extend_from_slice(&self.total_len.to_be_bytes());
        w.extend_from_slice(&self.chunk_size.to_be_bytes());
        w.extend_from_slice(&self.chunk_count.to_be_bytes());
        w.extend_from_slice(&self.chunks_root);
        if w.len() > mini_bearer::MAX_CHANNEL_PLAINTEXT_BYTES {
            return Err(ConsensusError::TooLarge);
        }
        Ok(w)
    }

    pub fn from_wire_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > mini_bearer::MAX_CHANNEL_PLAINTEXT_BYTES {
            return Err(ConsensusError::TooLarge);
        }
        let mut r = Reader::new(bytes);
        if r.take(DOMAIN.len())? != DOMAIN {
            return Err(ConsensusError::Malformed);
        }
        let header = decode_header(&mut r)?;
        let qc = decode_qc(&mut r)?;
        let total_len = r.u64()?;
        let chunk_size = r.u32()?;
        let chunk_count = r.u32()?;
        let mut chunks_root = [0u8; 32];
        chunks_root.copy_from_slice(r.take(32)?);
        if !r.finished() {
            return Err(ConsensusError::Malformed);
        }
        let manifest = Self {
            header,
            qc,
            total_len,
            chunk_size,
            chunk_count,
            chunks_root,
        };
        manifest.check_shape()?;
        if manifest.to_wire_bytes()?.as_slice() != bytes {
            return Err(ConsensusError::Malformed);
        }
        Ok(manifest)
    }

    /// Internal shape consistency: chunk size/count/total_len must actually
    /// agree, and stay within this module's bounds — independent of whether
    /// the header, QC, or `chunks_root` are honest.
    fn check_shape(&self) -> Result<()> {
        if self.total_len == 0
            || (self.chunk_size as usize) < MIN_CHUNK_BYTES
            || self.chunk_size as usize > MAX_CHUNK_BYTES
            || self.chunk_count == 0
            || self.chunk_count as usize > MAX_CHUNKS
        {
            return Err(ConsensusError::Malformed);
        }
        let expected_count = self.total_len.div_ceil(self.chunk_size as u64);
        if expected_count != self.chunk_count as u64 {
            return Err(ConsensusError::Malformed);
        }
        Ok(())
    }

    /// Exact length of the chunk at `index`, accounting for a shorter final
    /// chunk. Callers must have already checked `index < chunk_count`.
    fn chunk_len(&self, index: u32) -> usize {
        if index as u64 == self.chunk_count as u64 - 1 {
            let full_chunks_len = self.chunk_size as u64 * (self.chunk_count as u64 - 1);
            (self.total_len - full_chunks_len) as usize
        } else {
            self.chunk_size as usize
        }
    }
}

/// One chunk of a manifest's state bytes, plus its membership proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkResponse {
    pub index: u32,
    pub data: Vec<u8>,
    pub proof: ChunkProof,
}

impl ChunkResponse {
    pub fn to_wire_bytes(&self) -> Result<Vec<u8>> {
        let mut w = Vec::new();
        w.extend_from_slice(DOMAIN);
        w.extend_from_slice(&self.index.to_be_bytes());
        put_bytes(&mut w, &self.data);
        w.extend_from_slice(&self.proof.to_wire_bytes());
        if w.len() > mini_bearer::MAX_CHANNEL_PLAINTEXT_BYTES {
            return Err(ConsensusError::TooLarge);
        }
        Ok(w)
    }

    pub fn from_wire_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > mini_bearer::MAX_CHANNEL_PLAINTEXT_BYTES {
            return Err(ConsensusError::TooLarge);
        }
        let mut r = Reader::new(bytes);
        if r.take(DOMAIN.len())? != DOMAIN {
            return Err(ConsensusError::Malformed);
        }
        let index = r.u32()?;
        let data = r.bytes(MAX_CHUNK_BYTES)?.to_vec();
        let proof = ChunkProof::from_wire_bytes(&mut r)?;
        if !r.finished() {
            return Err(ConsensusError::Malformed);
        }
        Ok(Self { index, data, proof })
    }
}

/// Ask a specific peer for one chunk of the snapshot at `height` on
/// `network_id`. Carried over real TCP by [`crate::net::chunk_sync_over_tcp`]/
/// [`crate::net::serve_chunk_sync_over_tcp`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkRequest {
    pub network_id: [u8; 32],
    pub height: u64,
    pub index: u32,
}

impl ChunkRequest {
    pub fn to_wire_bytes(&self) -> Vec<u8> {
        let mut w = Vec::with_capacity(DOMAIN.len() + 44);
        w.extend_from_slice(DOMAIN);
        w.extend_from_slice(&self.network_id);
        w.extend_from_slice(&self.height.to_be_bytes());
        w.extend_from_slice(&self.index.to_be_bytes());
        w
    }

    pub fn from_wire_bytes(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        if r.take(DOMAIN.len())? != DOMAIN {
            return Err(ConsensusError::Malformed);
        }
        let mut network_id = [0u8; 32];
        network_id.copy_from_slice(r.take(32)?);
        let height = r.u64()?;
        let index = r.u32()?;
        if !r.finished() {
            return Err(ConsensusError::Malformed);
        }
        Ok(Self {
            network_id,
            height,
            index,
        })
    }
}

/// Ask a peer for the chunked-transfer manifest of its current latest
/// finalized snapshot on `network_id`, split into `chunk_size`-byte pieces.
/// `chunk_size` is the *requester's* choice — a weak or lossy-linked caller
/// asks for something small (down to [`MIN_CHUNK_BYTES`]) regardless of how
/// large the serving peer's own state or link happens to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManifestRequest {
    pub network_id: [u8; 32],
    pub chunk_size: u32,
}

impl ManifestRequest {
    pub fn to_wire_bytes(&self) -> Vec<u8> {
        let mut w = Vec::with_capacity(DOMAIN.len() + 36);
        w.extend_from_slice(DOMAIN);
        w.extend_from_slice(&self.network_id);
        w.extend_from_slice(&self.chunk_size.to_be_bytes());
        w
    }

    pub fn from_wire_bytes(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        if r.take(DOMAIN.len())? != DOMAIN {
            return Err(ConsensusError::Malformed);
        }
        let mut network_id = [0u8; 32];
        network_id.copy_from_slice(r.take(32)?);
        let chunk_size = r.u32()?;
        if !r.finished() {
            return Err(ConsensusError::Malformed);
        }
        Ok(Self {
            network_id,
            chunk_size,
        })
    }
}

const MANIFEST_RESPONSE_TAG_WRONG_NETWORK: u8 = 0;
const MANIFEST_RESPONSE_TAG_UNAVAILABLE: u8 = 1;
const MANIFEST_RESPONSE_TAG_MANIFEST: u8 = 2;

/// What a peer can supply for one [`ManifestRequest`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestResponse {
    /// The request names a different settlement/consensus network.
    WrongNetwork,
    /// The peer retains no finalized snapshot at all yet.
    Unavailable,
    /// The peer's current chunked-transfer manifest. Boxed so the control
    /// variants above do not pay the manifest's size on every response
    /// value (mirrors [`crate::state_sync::StateSyncPayload::Snapshot`]).
    Manifest(Box<SnapshotManifest>),
}

impl ManifestResponse {
    pub fn to_wire_bytes(&self) -> Result<Vec<u8>> {
        let mut w = Vec::new();
        w.extend_from_slice(DOMAIN);
        match self {
            ManifestResponse::WrongNetwork => w.push(MANIFEST_RESPONSE_TAG_WRONG_NETWORK),
            ManifestResponse::Unavailable => w.push(MANIFEST_RESPONSE_TAG_UNAVAILABLE),
            ManifestResponse::Manifest(manifest) => {
                w.push(MANIFEST_RESPONSE_TAG_MANIFEST);
                put_bytes(&mut w, &manifest.to_wire_bytes()?);
            }
        }
        if w.len() > mini_bearer::MAX_CHANNEL_PLAINTEXT_BYTES {
            return Err(ConsensusError::TooLarge);
        }
        Ok(w)
    }

    pub fn from_wire_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > mini_bearer::MAX_CHANNEL_PLAINTEXT_BYTES {
            return Err(ConsensusError::TooLarge);
        }
        let mut r = Reader::new(bytes);
        if r.take(DOMAIN.len())? != DOMAIN {
            return Err(ConsensusError::Malformed);
        }
        let response = match r.u8()? {
            MANIFEST_RESPONSE_TAG_WRONG_NETWORK => ManifestResponse::WrongNetwork,
            MANIFEST_RESPONSE_TAG_UNAVAILABLE => ManifestResponse::Unavailable,
            MANIFEST_RESPONSE_TAG_MANIFEST => {
                let manifest_bytes = r.bytes(mini_bearer::MAX_CHANNEL_PLAINTEXT_BYTES)?;
                ManifestResponse::Manifest(Box::new(SnapshotManifest::from_wire_bytes(
                    manifest_bytes,
                )?))
            }
            _ => return Err(ConsensusError::Malformed),
        };
        if !r.finished() {
            return Err(ConsensusError::Malformed);
        }
        Ok(response)
    }
}

/// Splits one [`ConsensusSnapshot`] into Merkle-authenticated chunks and
/// serves them on demand. Construction pays the encode-and-build-tree cost
/// once; [`Self::chunk`] afterward is a slice copy plus one proof lookup.
#[derive(Debug)]
pub struct SnapshotChunker {
    manifest: SnapshotManifest,
    state_bytes: Vec<u8>,
    levels: Vec<Vec<[u8; 32]>>,
}

impl SnapshotChunker {
    pub fn new(snapshot: &ConsensusSnapshot, chunk_size: usize) -> Result<Self> {
        if !(MIN_CHUNK_BYTES..=MAX_CHUNK_BYTES).contains(&chunk_size) {
            return Err(ConsensusError::Malformed);
        }
        let state_bytes = snapshot
            .state
            .to_snapshot_bytes()
            .map_err(ConsensusError::Execution)?;
        let total_len = state_bytes.len() as u64;
        if total_len == 0 {
            return Err(ConsensusError::Malformed);
        }
        let chunk_count = total_len.div_ceil(chunk_size as u64);
        if chunk_count > MAX_CHUNKS as u64 {
            return Err(ConsensusError::TooLarge);
        }
        let leaves: Vec<[u8; 32]> = state_bytes.chunks(chunk_size).map(leaf_hash).collect();
        let levels = build_levels(leaves);
        let chunks_root = levels.last().expect("levels is never empty")[0];
        let manifest = SnapshotManifest {
            header: snapshot.header.clone(),
            qc: snapshot.qc.clone(),
            total_len,
            chunk_size: chunk_size as u32,
            chunk_count: chunk_count as u32,
            chunks_root,
        };
        Ok(Self {
            manifest,
            state_bytes,
            levels,
        })
    }

    pub fn manifest(&self) -> &SnapshotManifest {
        &self.manifest
    }

    /// The chunk at `index` plus its membership proof, or `None` if out of
    /// range.
    pub fn chunk(&self, index: u32) -> Option<ChunkResponse> {
        if index >= self.manifest.chunk_count {
            return None;
        }
        let chunk_size = self.manifest.chunk_size as usize;
        let start = index as usize * chunk_size;
        let end = (start + chunk_size).min(self.state_bytes.len());
        let data = self.state_bytes[start..end].to_vec();
        let proof = prove(&self.levels, index as usize);
        Some(ChunkResponse { index, data, proof })
    }
}

/// Reassembles one [`SnapshotManifest`]'s chunks, verifying every chunk
/// against the manifest's `chunks_root` as it arrives, and only trusting
/// the fully reassembled state the same way `ConsensusSnapshot::
/// from_wire_bytes` already does — `header.state_root == state.commitment()`,
/// checked once, in [`Self::finish`].
#[derive(Debug)]
pub struct SnapshotAssembler {
    manifest: SnapshotManifest,
    chunks: Vec<Option<Vec<u8>>>,
    received: usize,
}

impl SnapshotAssembler {
    /// Begin assembling `manifest`. Verifies the manifest's QC is finalized
    /// under `validators`/`oracle` immediately, so a receiver never spends
    /// bandwidth fetching chunks for a snapshot whose finality proof was
    /// never going to hold up — the exact same [`mini_chain::verify_finality`]
    /// call [`Self::finish`] repeats at the end (via `ConsensusSnapshot::
    /// into_chain`); this is a fail-fast duplicate of that same check, not a
    /// separate trust decision.
    pub fn new(
        manifest: SnapshotManifest,
        validators: &ValidatorSet,
        oracle: &dyn ValidatorOracle,
    ) -> Result<Self> {
        mini_chain::verify_finality(&manifest.qc, validators, oracle)
            .map_err(ConsensusError::Chain)?;
        if manifest.header.height == 0
            || manifest.qc.height != manifest.header.height
            || manifest.qc.block_hash != manifest.header.hash()
        {
            return Err(ConsensusError::SnapshotProofMismatch);
        }
        let chunk_count = manifest.chunk_count as usize;
        Ok(Self {
            manifest,
            chunks: vec![None; chunk_count],
            received: 0,
        })
    }

    pub fn manifest(&self) -> &SnapshotManifest {
        &self.manifest
    }

    /// Indices not yet accepted, in ascending order — what a caller should
    /// still request, from this peer or another.
    pub fn missing_indices(&self) -> Vec<u32> {
        self.chunks
            .iter()
            .enumerate()
            .filter_map(|(i, c)| c.is_none().then_some(i as u32))
            .collect()
    }

    pub fn is_complete(&self) -> bool {
        self.received == self.chunks.len()
    }

    /// Verify and accept one chunk. Returns whether every chunk has now
    /// been accepted. A chunk whose length disagrees with the manifest's
    /// declared size for that index, or whose proof does not verify against
    /// `chunks_root`, is rejected and never stored — the caller should ask
    /// a different peer for that index rather than retry the same one
    /// blindly. Accepting the same already-filled index again is a no-op
    /// success, not an error, so a slow second reply from a retried request
    /// cannot fail an otherwise-complete assembly.
    pub fn accept_chunk(&mut self, response: &ChunkResponse) -> Result<bool> {
        if response.index >= self.manifest.chunk_count {
            return Err(ConsensusError::Malformed);
        }
        let slot_index = response.index as usize;
        if self.chunks[slot_index].is_some() {
            return Ok(self.is_complete());
        }
        if response.data.len() != self.manifest.chunk_len(response.index) {
            return Err(ConsensusError::Malformed);
        }
        if !response.proof.verify(
            &response.data,
            slot_index,
            self.manifest.chunk_count as usize,
            self.manifest.chunks_root,
        ) {
            return Err(ConsensusError::SnapshotProofMismatch);
        }
        self.chunks[slot_index] = Some(response.data.clone());
        self.received += 1;
        Ok(self.is_complete())
    }

    /// Reassemble every accepted chunk in order, decode the execution state,
    /// and structurally check it against the manifest's header/QC exactly as
    /// `ConsensusSnapshot::from_wire_bytes` already does for a single-frame
    /// snapshot — `header.state_root == state.commitment()`, `qc.height ==
    /// header.height`, `qc.block_hash == header.hash()`. Finality itself
    /// (`>2/3` of `validators`) was already checked in [`Self::new`]; this
    /// does not re-check it.
    ///
    /// Returns a plain [`ConsensusSnapshot`] rather than a live
    /// `LedgerChain` — a caller with a `mini_chain::ValidatorSet`/
    /// `ValidatorOracle` still in scope can turn it into one with
    /// `ConsensusSnapshot::into_chain` unchanged (as `crate::net::
    /// chunk_sync_over_tcp` does, via `ConsensusNode::apply_state_sync`,
    /// which performs that same finality/state verification again itself —
    /// exactly the layered "peer supplies bytes, never trust" discipline
    /// `state_sync_over_tcp` already follows).
    ///
    /// Errors [`ConsensusError::Malformed`] if any chunk is still missing.
    pub fn finish(self) -> Result<ConsensusSnapshot> {
        if !self.is_complete() {
            return Err(ConsensusError::Malformed);
        }
        let mut state_bytes = Vec::with_capacity(self.manifest.total_len as usize);
        for chunk in self.chunks {
            state_bytes.extend_from_slice(&chunk.expect("is_complete just checked every slot"));
        }
        let state =
            LedgerState::from_snapshot_bytes(&state_bytes).map_err(ConsensusError::Execution)?;
        ConsensusSnapshot::new(self.manifest.header, self.manifest.qc, state)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use did_mini::{Capabilities, Controller, Did, Kel};
    use mini_chain::{sign_vote, VoteKind};
    use mini_crypto::SigningKey;
    use mini_economy::Amount;
    use mini_execution::LedgerState;

    use super::*;

    #[derive(Default)]
    struct Directory(BTreeMap<String, Kel>);

    impl ValidatorOracle for Directory {
        fn kel(&self, did: &Did) -> Option<&Kel> {
            self.0.get(did.scid())
        }
    }

    /// A finalized, QC-bound snapshot over a non-trivially-sized state (100
    /// funded accounts, each a real Ed25519 key -- the only account shape
    /// `LedgerState` accepts), so its encoded bytes reliably span several
    /// `MIN_CHUNK_BYTES`-sized chunks. Also returns the validator set/oracle
    /// that can verify its QC.
    fn snapshot_and_validators() -> (ConsensusSnapshot, ValidatorSet, Directory) {
        let mut directory = Directory::default();
        let mut allocations = Vec::new();
        let mut total = Amount::ZERO;
        for i in 0u8..100 {
            let account = SigningKey::from_seed(&[i; 32]).verifying_key().to_bytes();
            let amount = Amount::from_micro(1_000 + i as u128);
            total = total.checked_add(amount).unwrap();
            allocations.push((account, amount));
        }
        let state = LedgerState::with_genesis_balances(total, allocations).unwrap();
        let header = BlockHeader {
            height: 1,
            prev_hash: [0; 32],
            state_root: state.commitment(),
            body_root: mini_execution::SettlementBlockBody::new(Vec::new()).hash(),
            timestamp_ms: 1,
            proposer: Controller::incept_single_from_seeds(&[90; 32], &[91; 32])
                .unwrap()
                .did(),
        };
        let block_hash = header.hash();
        let mut roots = Vec::new();
        let mut votes = Vec::new();
        for seed in [10u8, 20, 30, 40] {
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
            roots.push(root.did());
            directory
                .0
                .insert(root.did().scid().to_string(), root.kel());
            directory
                .0
                .insert(device.did().scid().to_string(), device.kel());
            votes.push(sign_vote(
                VoteKind::Precommit,
                1,
                0,
                block_hash,
                &root.did(),
                &device,
            ));
        }
        let validators = ValidatorSet::new(roots).unwrap();
        let qc = QuorumCertificate {
            height: 1,
            round: 0,
            block_hash,
            votes,
        };
        (
            ConsensusSnapshot::new(header, qc, state).unwrap(),
            validators,
            directory,
        )
    }

    #[test]
    fn a_manifest_round_trips_over_the_wire() {
        let (snapshot, _, _) = snapshot_and_validators();
        let chunker = SnapshotChunker::new(&snapshot, MIN_CHUNK_BYTES).unwrap();
        let bytes = chunker.manifest().to_wire_bytes().unwrap();
        let decoded = SnapshotManifest::from_wire_bytes(&bytes).unwrap();
        assert_eq!(&decoded, chunker.manifest());
        assert!(decoded.chunk_count > 1, "fixture state must span >1 chunk");
    }

    #[test]
    fn full_chunk_by_chunk_transfer_reassembles_into_the_same_chain() {
        let (snapshot, validators, directory) = snapshot_and_validators();
        let expected_commitment = snapshot.state.commitment();
        let chunker = SnapshotChunker::new(&snapshot, MIN_CHUNK_BYTES).unwrap();
        let manifest = chunker.manifest().clone();
        assert!(manifest.chunk_count > 1, "fixture state must span >1 chunk");

        let mut assembler =
            SnapshotAssembler::new(manifest.clone(), &validators, &directory).unwrap();
        for index in 0..manifest.chunk_count {
            let response = chunker.chunk(index).unwrap();
            // Every response round-trips over the wire exactly like any
            // other consensus message before a receiver ever sees it.
            let response =
                ChunkResponse::from_wire_bytes(&response.to_wire_bytes().unwrap()).unwrap();
            let complete = assembler.accept_chunk(&response).unwrap();
            assert_eq!(complete, index + 1 == manifest.chunk_count);
        }
        assert!(assembler.is_complete());

        let reassembled = assembler.finish().unwrap();
        assert_eq!(reassembled.state.commitment(), expected_commitment);
        let chain = reassembled
            .into_chain(mini_settlement::MININET_NETWORK_ID, &validators, &directory)
            .unwrap();
        assert_eq!(chain.height(), 1);
        assert_eq!(chain.state().commitment(), expected_commitment);
    }

    #[test]
    fn chunks_may_arrive_out_of_order_and_re_delivery_is_harmless() {
        let (snapshot, validators, directory) = snapshot_and_validators();
        let chunker = SnapshotChunker::new(&snapshot, MIN_CHUNK_BYTES).unwrap();
        let manifest = chunker.manifest().clone();
        assert!(
            manifest.chunk_count >= 3,
            "fixture state must span >=3 chunks"
        );

        let mut assembler =
            SnapshotAssembler::new(manifest.clone(), &validators, &directory).unwrap();
        let last = manifest.chunk_count - 1;
        assembler
            .accept_chunk(&chunker.chunk(last).unwrap())
            .unwrap();
        assembler.accept_chunk(&chunker.chunk(0).unwrap()).unwrap();
        // Redeliver an already-accepted chunk: harmless no-op, not an error.
        assembler.accept_chunk(&chunker.chunk(0).unwrap()).unwrap();
        for index in 1..last {
            assembler
                .accept_chunk(&chunker.chunk(index).unwrap())
                .unwrap();
        }
        assert!(assembler.is_complete());
        assembler.finish().unwrap();
    }

    #[test]
    fn missing_indices_reports_exactly_what_is_still_needed() {
        let (snapshot, validators, directory) = snapshot_and_validators();
        let chunker = SnapshotChunker::new(&snapshot, MIN_CHUNK_BYTES).unwrap();
        let manifest = chunker.manifest().clone();
        assert!(manifest.chunk_count >= 3);

        let mut assembler =
            SnapshotAssembler::new(manifest.clone(), &validators, &directory).unwrap();
        assembler.accept_chunk(&chunker.chunk(1).unwrap()).unwrap();
        let missing = assembler.missing_indices();
        assert!(!missing.contains(&1));
        assert_eq!(missing.len(), manifest.chunk_count as usize - 1);
    }

    #[test]
    fn a_tampered_chunk_fails_its_own_proof() {
        let (snapshot, validators, directory) = snapshot_and_validators();
        let chunker = SnapshotChunker::new(&snapshot, MIN_CHUNK_BYTES).unwrap();
        let manifest = chunker.manifest().clone();

        let mut assembler = SnapshotAssembler::new(manifest, &validators, &directory).unwrap();
        let mut tampered = chunker.chunk(0).unwrap();
        if let Some(byte) = tampered.data.first_mut() {
            *byte ^= 1;
        } else {
            tampered.data.push(1);
        }
        assert_eq!(
            assembler.accept_chunk(&tampered),
            Err(ConsensusError::SnapshotProofMismatch)
        );
        assert!(!assembler.is_complete());
    }

    #[test]
    fn a_proof_from_the_wrong_index_does_not_verify() {
        let (snapshot, validators, directory) = snapshot_and_validators();
        let chunker = SnapshotChunker::new(&snapshot, MIN_CHUNK_BYTES).unwrap();
        let manifest = chunker.manifest().clone();
        assert!(manifest.chunk_count >= 2);

        let mut assembler = SnapshotAssembler::new(manifest, &validators, &directory).unwrap();
        let chunk0 = chunker.chunk(0).unwrap();
        let chunk1 = chunker.chunk(1).unwrap();
        // Splice chunk 1's data under chunk 0's proof/index.
        let mismatched = ChunkResponse {
            index: 0,
            data: chunk1.data,
            proof: chunk0.proof,
        };
        assert_eq!(
            assembler.accept_chunk(&mismatched),
            Err(ConsensusError::SnapshotProofMismatch)
        );
    }

    #[test]
    fn a_short_final_chunk_is_the_exact_remainder_not_padded_or_truncated() {
        let (snapshot, _, _) = snapshot_and_validators();
        let chunker = SnapshotChunker::new(&snapshot, MIN_CHUNK_BYTES).unwrap();
        let manifest = chunker.manifest();
        let last = manifest.chunk_count - 1;
        let last_chunk = chunker.chunk(last).unwrap();
        let full_len = manifest.chunk_size as u64 * last as u64;
        assert_eq!(last_chunk.data.len() as u64, manifest.total_len - full_len);
        assert!(last_chunk.data.len() as u32 <= manifest.chunk_size);
    }

    #[test]
    fn finish_before_every_chunk_is_accepted_is_rejected() {
        let (snapshot, validators, directory) = snapshot_and_validators();
        let chunker = SnapshotChunker::new(&snapshot, MIN_CHUNK_BYTES).unwrap();
        let manifest = chunker.manifest().clone();
        assert!(manifest.chunk_count > 1);

        let mut assembler = SnapshotAssembler::new(manifest, &validators, &directory).unwrap();
        assembler.accept_chunk(&chunker.chunk(0).unwrap()).unwrap();
        assert_eq!(assembler.finish().unwrap_err(), ConsensusError::Malformed);
    }

    #[test]
    fn a_manifest_with_a_non_quorate_qc_is_rejected_before_any_chunk_is_fetched() {
        let (snapshot, validators, directory) = snapshot_and_validators();
        let chunker = SnapshotChunker::new(&snapshot, MIN_CHUNK_BYTES).unwrap();
        let mut manifest = chunker.manifest().clone();
        manifest.qc.votes.clear();
        assert!(SnapshotAssembler::new(manifest, &validators, &directory).is_err());
    }

    #[test]
    fn a_chunk_index_out_of_range_is_rejected() {
        let (snapshot, validators, directory) = snapshot_and_validators();
        let chunker = SnapshotChunker::new(&snapshot, MIN_CHUNK_BYTES).unwrap();
        let manifest = chunker.manifest().clone();
        let out_of_range = manifest.chunk_count;
        let mut assembler = SnapshotAssembler::new(manifest, &validators, &directory).unwrap();
        let bogus = ChunkResponse {
            index: out_of_range,
            data: vec![0; MIN_CHUNK_BYTES],
            proof: ChunkProof { siblings: vec![] },
        };
        assert_eq!(
            assembler.accept_chunk(&bogus),
            Err(ConsensusError::Malformed)
        );
        assert!(chunker.chunk(out_of_range).is_none());
    }

    #[test]
    fn a_manifest_whose_declared_chunk_count_disagrees_with_its_own_math_is_rejected() {
        let (snapshot, _, _) = snapshot_and_validators();
        let chunker = SnapshotChunker::new(&snapshot, MIN_CHUNK_BYTES).unwrap();
        let mut manifest = chunker.manifest().clone();
        manifest.chunk_count += 1;
        assert_eq!(
            SnapshotManifest::from_wire_bytes(&manifest.to_wire_bytes().unwrap()),
            Err(ConsensusError::Malformed)
        );
    }

    #[test]
    fn chunk_size_outside_the_allowed_range_is_rejected() {
        let (snapshot, _, _) = snapshot_and_validators();
        assert!(SnapshotChunker::new(&snapshot, MIN_CHUNK_BYTES - 1).is_err());
        assert!(SnapshotChunker::new(&snapshot, MAX_CHUNK_BYTES + 1).is_err());
    }

    #[test]
    fn a_chunk_request_round_trips_and_rejects_trailing_bytes() {
        let request = ChunkRequest {
            network_id: [7; 32],
            height: 42,
            index: 3,
        };
        let bytes = request.to_wire_bytes();
        assert_eq!(ChunkRequest::from_wire_bytes(&bytes).unwrap(), request);
        let mut trailing = bytes;
        trailing.push(0);
        assert!(ChunkRequest::from_wire_bytes(&trailing).is_err());
    }

    #[test]
    fn truncation_of_a_chunk_response_at_every_length_is_rejected_never_panics() {
        let (snapshot, _, _) = snapshot_and_validators();
        let chunker = SnapshotChunker::new(&snapshot, MIN_CHUNK_BYTES).unwrap();
        let bytes = chunker.chunk(0).unwrap().to_wire_bytes().unwrap();
        for cut in 0..bytes.len() {
            assert!(ChunkResponse::from_wire_bytes(&bytes[..cut]).is_err());
        }
    }

    #[test]
    fn a_single_chunk_snapshot_still_produces_a_valid_zero_depth_proof() {
        let (snapshot, validators, directory) = snapshot_and_validators();
        // A chunk size at least as large as the whole encoded state yields
        // exactly one chunk and a zero-length proof.
        let state_len = snapshot.state.to_snapshot_bytes().unwrap().len();
        let chunk_size = state_len.max(MIN_CHUNK_BYTES);
        let chunker = SnapshotChunker::new(&snapshot, chunk_size).unwrap();
        assert_eq!(chunker.manifest().chunk_count, 1);

        let mut assembler =
            SnapshotAssembler::new(chunker.manifest().clone(), &validators, &directory).unwrap();
        assert!(assembler.accept_chunk(&chunker.chunk(0).unwrap()).unwrap());
        assembler.finish().unwrap();
    }
}
