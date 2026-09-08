//! Nested manifests for payloads beyond one manifest's ≤256-chunk cap
//! (D-0419) -- the "later batch" this crate's own module doc named.
//!
//! A [`Superblock`] is one signed object recording the whole payload's
//! total length and BLAKE3 digest, plus an ordered list of ordinary
//! [`crate::Manifest`]s ("parts"). Each part is published exactly the way
//! [`crate::publish_media`] already does -- chunked, digest-checked,
//! content-addressed -- so a superblock adds exactly one more composition
//! level, not a second storage model. [`assemble_superblock`]
//! re-verifies every part's own digest (via [`crate::assemble`]) *and*
//! the whole concatenation against the superblock's own recorded digest,
//! so a mix of validly-signed but unrelated parts is still caught.
//!
//! One level of nesting only, per this crate's own stated scope ("nests
//! manifests," singular). A superblock addresses up to
//! [`MAX_PARTS`] × [`crate::MAX_TOTAL_LEN`] bytes; deeper nesting
//! (superblocks of superblocks) is not built and is later, separately
//! -scoped work if this bound ever proves insufficient.

use did_mini::{Controller, Did};
use mini_crypto::HashAlgorithm;
use mini_objects::{Object, ObjectBuilder, ObjectId, ObjectType, Payload, MAX_LINKS};
use mini_store::{Backend, Store};

use crate::{
    assemble, assemble_to_writer, missing_chunks, publish_media, read_manifest, MediaError, Result,
    CHUNK_SIZE, MAX_CHUNKS, MAX_CONTENT_TYPE_BYTES, MAX_TOTAL_LEN,
};

/// The custom object type carrying a [`Superblock`].
pub const SUPERBLOCK_TYPE: &str = "mini/superblock";
/// Maximum parts one superblock may address (envelope link cap, same
/// bound [`crate::MAX_CHUNKS`] already uses for one manifest's chunks).
pub const MAX_PARTS: usize = MAX_LINKS;
/// Maximum total payload one superblock may declare (allocation bound for
/// untrusted superblocks).
pub const MAX_SUPERBLOCK_TOTAL_LEN: u64 = (MAX_PARTS as u64) * MAX_TOTAL_LEN;

/// Parsed superblock metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Superblock {
    /// The superblock object's id.
    pub id: ObjectId,
    /// MIME-style content type.
    pub content_type: String,
    /// Total payload length in bytes, across every part.
    pub total_len: u64,
    /// BLAKE3 digest of the whole payload (all parts concatenated).
    pub digest: [u8; 32],
    /// Ordered part manifest ids.
    pub parts: Vec<ObjectId>,
}

/// Split `bytes` into one or more manifest-sized parts (each up to
/// `chunks_per_part` chunks, i.e. `chunks_per_part * CHUNK_SIZE` bytes),
/// publish each part exactly as [`crate::publish_media`] already does,
/// and wrap them in one signed superblock recording the whole payload's
/// length and digest. `chunks_per_part` must be in `1..=MAX_CHUNKS`;
/// callers with no specific transport-granularity reason should pass
/// `MAX_CHUNKS` for the fewest parts.
#[allow(clippy::too_many_arguments)]
pub fn publish_large_media<B: Backend>(
    store: &mut Store<B>,
    human: &Did,
    device: &Controller,
    content_type: &str,
    bytes: &[u8],
    chunks_per_part: usize,
    timestamp_ms: u64,
    sequence: u64,
) -> Result<Superblock> {
    if chunks_per_part == 0 || chunks_per_part > MAX_CHUNKS {
        return Err(MediaError::FieldTooLarge);
    }
    if content_type.len() > MAX_CONTENT_TYPE_BYTES {
        return Err(MediaError::FieldTooLarge);
    }
    let part_len = chunks_per_part * CHUNK_SIZE;
    let n_parts = if bytes.is_empty() {
        1
    } else {
        bytes.len().div_ceil(part_len)
    };
    if n_parts > MAX_PARTS {
        return Err(MediaError::TooLarge);
    }

    // Distinct, non-overlapping sequence ranges per part -- not
    // correctness-critical (content addressing already makes distinct
    // bytes produce distinct ids regardless), but keeps provenance
    // sensible the same way `publish_media`'s own per-chunk offsets do.
    let seq_stride = (chunks_per_part as u64).saturating_add(1);
    let mut part_ids = Vec::with_capacity(n_parts);
    if bytes.is_empty() {
        let manifest = publish_media(
            store,
            human,
            device,
            content_type,
            b"",
            timestamp_ms,
            sequence,
        )?;
        part_ids.push(manifest.id);
    } else {
        for (i, part_bytes) in bytes.chunks(part_len).enumerate() {
            let manifest = publish_media(
                store,
                human,
                device,
                content_type,
                part_bytes,
                timestamp_ms,
                sequence.wrapping_add(i as u64 * seq_stride),
            )?;
            part_ids.push(manifest.id);
        }
    }

    let digest = HashAlgorithm::Blake3.digest(bytes);
    let mut payload = Vec::new();
    payload.extend_from_slice(&(content_type.len() as u32).to_be_bytes());
    payload.extend_from_slice(content_type.as_bytes());
    payload.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    payload.extend_from_slice(&digest);

    let mut builder = ObjectBuilder::new(ObjectType::Custom(SUPERBLOCK_TYPE.to_string()))
        .timestamp_ms(timestamp_ms)
        .sequence(sequence)
        .payload(Payload::Public(payload));
    for p in &part_ids {
        builder = builder.link("part", p.clone());
    }
    let superblock = builder.sign(human, device)?;
    store.insert(&superblock)?;

    Ok(Superblock {
        id: superblock.id().clone(),
        content_type: content_type.to_string(),
        total_len: bytes.len() as u64,
        digest,
        parts: part_ids,
    })
}

/// Parse a superblock object.
pub fn read_superblock(obj: &Object) -> Result<Superblock> {
    if obj.object_type != ObjectType::Custom(SUPERBLOCK_TYPE.to_string()) {
        return Err(MediaError::BadManifest);
    }
    let b = match &obj.payload {
        Payload::Public(b) => b,
        Payload::Encrypted(_) => return Err(MediaError::BadManifest),
    };
    if b.len() < 4 {
        return Err(MediaError::BadManifest);
    }
    let ct_len = u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize;
    if ct_len > MAX_CONTENT_TYPE_BYTES || b.len() < 4 + ct_len + 8 + 32 {
        return Err(MediaError::BadManifest);
    }
    let content_type =
        String::from_utf8(b[4..4 + ct_len].to_vec()).map_err(|_| MediaError::BadManifest)?;
    let mut off = 4 + ct_len;
    let mut len8 = [0u8; 8];
    len8.copy_from_slice(&b[off..off + 8]);
    off += 8;
    let mut digest = [0u8; 32];
    digest.copy_from_slice(&b[off..off + 32]);
    off += 32;
    if off != b.len() {
        return Err(MediaError::BadManifest);
    }

    let parts: Vec<ObjectId> = obj
        .links
        .iter()
        .filter(|l| l.rel == "part")
        .map(|l| l.target.clone())
        .collect();
    if parts.is_empty() || parts.len() > MAX_PARTS {
        return Err(MediaError::BadManifest);
    }
    let total_len = u64::from_be_bytes(len8);
    if total_len > MAX_SUPERBLOCK_TOTAL_LEN || total_len > (parts.len() as u64) * MAX_TOTAL_LEN {
        return Err(MediaError::BadManifest);
    }
    Ok(Superblock {
        id: obj.id().clone(),
        content_type,
        total_len,
        digest,
        parts,
    })
}

/// What a caller still needs to fully materialize `superblock`: for each
/// part manifest not yet held, the manifest object itself (its chunk list
/// can't be inspected until it arrives); for each part manifest already
/// held, whatever chunks that part is still missing.
pub fn missing_superblock_chunks<B: Backend>(
    store: &Store<B>,
    superblock: &Superblock,
) -> Result<Vec<ObjectId>> {
    let mut out = Vec::new();
    for part_id in &superblock.parts {
        if !store.contains(part_id)? {
            out.push(part_id.clone());
            continue;
        }
        let manifest = read_manifest(&store.get(part_id)?)?;
        out.extend(missing_chunks(store, &manifest)?);
    }
    Ok(out)
}

/// Assemble the full payload from the store: reassembles and independently
/// digest-checks every part (via [`crate::assemble`]), then re-verifies
/// the whole concatenation against the superblock's own recorded length
/// and digest. Returns [`MediaError::Incomplete`] while any part manifest
/// or chunk is still missing.
pub fn assemble_superblock<B: Backend>(
    store: &Store<B>,
    superblock: &Superblock,
) -> Result<Vec<u8>> {
    let cap = superblock.total_len.min(MAX_SUPERBLOCK_TOTAL_LEN) as usize;
    let mut out: Vec<u8> = Vec::with_capacity(cap);
    for part_id in &superblock.parts {
        if !store.contains(part_id)? {
            return Err(MediaError::Incomplete);
        }
        let manifest = read_manifest(&store.get(part_id)?)?;
        let part_bytes = assemble(store, &manifest)?;
        if out.len() + part_bytes.len() > cap {
            return Err(MediaError::DigestMismatch);
        }
        out.extend_from_slice(&part_bytes);
    }
    if out.len() as u64 != superblock.total_len
        || HashAlgorithm::Blake3.digest(&out) != superblock.digest
    {
        return Err(MediaError::DigestMismatch);
    }
    Ok(out)
}

/// A `Write` adapter that forwards every byte to `inner` while incrementally
/// hashing everything that passes through, so a caller composing several
/// smaller streamed writes (one per part, here) can still check one running
/// digest across all of them without re-reading what was already written.
struct HashingWriter<'a, W> {
    inner: &'a mut W,
    hasher: mini_crypto::IncrementalHash,
    written: u64,
}

impl<'a, W: std::io::Write> HashingWriter<'a, W> {
    fn new(inner: &'a mut W) -> Self {
        Self {
            inner,
            hasher: HashAlgorithm::Blake3.incremental(),
            written: 0,
        }
    }

    fn finish(self) -> (u64, [u8; 32]) {
        (self.written, self.hasher.finalize())
    }
}

impl<W: std::io::Write> std::io::Write for HashingWriter<'_, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.hasher.update(&buf[..n]);
        self.written += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// Like [`assemble_superblock`], but streams the payload directly to
/// `writer` instead of building the whole (up to 64 GiB) concatenation as
/// one `Vec<u8>` first. Each part still only ever holds one chunk (at most
/// [`crate::CHUNK_SIZE`] bytes) in memory at a time, via
/// [`crate::assemble_to_writer`] — this function's own added memory cost is
/// the fixed size of one running hash.
pub fn assemble_superblock_to_writer<B: Backend, W: std::io::Write>(
    store: &Store<B>,
    superblock: &Superblock,
    writer: &mut W,
) -> Result<()> {
    let cap = superblock.total_len.min(MAX_SUPERBLOCK_TOTAL_LEN);
    let mut hashing = HashingWriter::new(writer);
    for part_id in &superblock.parts {
        if !store.contains(part_id)? {
            return Err(MediaError::Incomplete);
        }
        let manifest = read_manifest(&store.get(part_id)?)?;
        if hashing.written + manifest.total_len > cap {
            return Err(MediaError::DigestMismatch);
        }
        assemble_to_writer(store, &manifest, &mut hashing)?;
    }
    let (written, digest) = hashing.finish();
    if written != superblock.total_len || digest != superblock.digest {
        return Err(MediaError::DigestMismatch);
    }
    Ok(())
}
