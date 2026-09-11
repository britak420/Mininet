//! Minimal hand-rolled canonical encoding helpers, matching this
//! workspace's house style (no `serde` anywhere in this tree — see
//! `mini-pipeline-protocol`'s own docs for why: a hand-written canonical
//! form is unambiguous and has no derive-macro-driven schema drift to
//! reason about). Every custody wire type in this crate builds its
//! canonical bytes from these primitives directly, the same way
//! `mini-private-payment::codec` and `mini-mesh`'s own message framing do.

use crate::error::{CustodyError, Result};

/// Append a `u16` length-prefixed byte string.
pub(crate) fn push_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
    out.extend_from_slice(bytes);
}

/// Append a length-prefixed UTF-8 string.
pub(crate) fn push_str(out: &mut Vec<u8>, s: &str) {
    push_bytes(out, s.as_bytes());
}

/// Read a `u16` length-prefixed byte slice from `input`, advancing it.
pub(crate) fn read_bytes<'a>(input: &mut &'a [u8]) -> Result<&'a [u8]> {
    if input.len() < 2 {
        return Err(CustodyError::InvalidManifest("truncated length prefix"));
    }
    let len = u16::from_be_bytes([input[0], input[1]]) as usize;
    *input = &input[2..];
    if input.len() < len {
        return Err(CustodyError::InvalidManifest("truncated field"));
    }
    let (field, rest) = input.split_at(len);
    *input = rest;
    Ok(field)
}

/// Read a length-prefixed UTF-8 string.
pub(crate) fn read_str<'a>(input: &mut &'a [u8]) -> Result<&'a str> {
    let bytes = read_bytes(input)?;
    core::str::from_utf8(bytes).map_err(|_| CustodyError::InvalidManifest("non-UTF-8 string"))
}

/// Read a fixed-size array, advancing `input`.
pub(crate) fn read_array<const N: usize>(input: &mut &[u8]) -> Result<[u8; N]> {
    if input.len() < N {
        return Err(CustodyError::InvalidManifest("truncated fixed field"));
    }
    let (field, rest) = input.split_at(N);
    *input = rest;
    let mut out = [0u8; N];
    out.copy_from_slice(field);
    Ok(out)
}

/// Read a `u16`, advancing `input`.
pub(crate) fn read_u16(input: &mut &[u8]) -> Result<u16> {
    Ok(u16::from_be_bytes(read_array::<2>(input)?))
}

/// Read a `u32`, advancing `input`.
pub(crate) fn read_u32(input: &mut &[u8]) -> Result<u32> {
    Ok(u32::from_be_bytes(read_array::<4>(input)?))
}

/// Read a `u64`, advancing `input`.
pub(crate) fn read_u64(input: &mut &[u8]) -> Result<u64> {
    Ok(u64::from_be_bytes(read_array::<8>(input)?))
}

/// Domain-separated BLAKE3: `BLAKE3(domain || parts...)`. The same
/// prefix-domain-separation shape every other hash in this tree already
/// uses (`mini_mesh::message_id`, `mini_treasury::curve::hash_to_scalar`,
/// `mini_bearer`'s various channel-binding hashes) -- not BLAKE3's keyed
/// `derive_key` mode, which is a different (and here unnecessary)
/// construction.
pub(crate) fn domain_hash(domain: &[u8], parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    for part in parts {
        hasher.update(part);
    }
    *hasher.finalize().as_bytes()
}
