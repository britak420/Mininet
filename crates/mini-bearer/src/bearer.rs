//! The [`Bearer`] trait and length-prefix framing.
//!
//! A bearer moves opaque frames and knows nothing about identity or content. Some
//! bearers are message-oriented (BLE GATT writes, the in-process bearer) and need
//! no framing; byte-stream bearers (TCP, a serial relay) use [`encode_frame`] and
//! [`FrameReader`] to turn a stream back into discrete frames.

use crate::error::{BearerError, Result};

/// Hard cap on a single frame. Bearers reject anything larger before allocating.
pub const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;

/// Hard cap on bytes buffered by a stream frame reader.
///
/// A byte-stream adapter must drain complete frames as it feeds data. Keeping this
/// cap equal to one maximum-sized frame plus its length prefix prevents a hostile
/// peer from forcing unbounded growth before a frame is authenticated.
pub const MAX_STREAM_BUFFER_BYTES: usize = MAX_FRAME_BYTES + 4;

/// A duplex, frame-oriented, best-effort byte transport between two endpoints.
///
/// Frames are discrete opaque payloads. A lossy or reordering bearer may drop or
/// reorder them. The encrypted channel is intentionally in-order; higher layers
/// that need delay-tolerant sync must add retransmission/reordering above it.
pub trait Bearer {
    /// Send one frame to the peer.
    fn send(&mut self, frame: &[u8]) -> Result<()>;

    /// Receive the next frame, blocking until one arrives or the peer closes.
    fn recv(&mut self) -> Result<Vec<u8>>;

    /// Receive a frame if one is ready, otherwise `Ok(None)`.
    fn try_recv(&mut self) -> Result<Option<Vec<u8>>>;

    /// Upper bound on a single frame this bearer can actually carry, if it
    /// has one narrower than [`MAX_FRAME_BYTES`]. `None` (the default,
    /// correct for [`crate::TcpBearer`]/[`crate::InProcessBearer`]) means no
    /// bound beyond `MAX_FRAME_BYTES` itself. A chunked small-payload bearer
    /// like [`crate::AndroidBleBearer`] overrides this: its negotiated MTU
    /// limits how many chunks a `u16` chunk count can express
    /// ([`crate::ble::chunk_frame`]'s own `TooManyChunks` bound), which is
    /// typically far smaller than `MAX_FRAME_BYTES`. A caller that knows
    /// this bound can reject an oversized payload before ever calling
    /// [`Bearer::send`], instead of discovering the failure only after
    /// something upstream (e.g. [`crate::Channel::seal`]) has already
    /// committed state on the assumption the frame would actually go out.
    fn max_frame_bytes(&self) -> Option<usize> {
        None
    }
}

/// Lets a caller hold a heterogeneous set of live links — some in-process
/// (a test), some real BLE, some TCP — in one collection, e.g.
/// `Vec<Box<dyn Bearer + Send>>`. Dispatches straight through to the boxed
/// value; no behavior of its own.
impl Bearer for Box<dyn Bearer + Send> {
    fn send(&mut self, frame: &[u8]) -> Result<()> {
        (**self).send(frame)
    }

    fn recv(&mut self) -> Result<Vec<u8>> {
        (**self).recv()
    }

    fn try_recv(&mut self) -> Result<Option<Vec<u8>>> {
        (**self).try_recv()
    }

    fn max_frame_bytes(&self) -> Option<usize> {
        (**self).max_frame_bytes()
    }
}

/// Encode a payload as a length-prefixed frame (`u32` big-endian length + bytes)
/// for transport over a byte-stream bearer.
pub fn encode_frame(payload: &[u8]) -> Result<Vec<u8>> {
    if payload.len() > MAX_FRAME_BYTES {
        return Err(BearerError::FrameTooLarge {
            max: MAX_FRAME_BYTES,
            got: payload.len(),
        });
    }
    let mut out = Vec::with_capacity(4 + payload.len());
    out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    out.extend_from_slice(payload);
    Ok(out)
}

/// Reassembles length-prefixed frames from a byte stream that may arrive in
/// arbitrary chunks.
#[derive(Debug, Default)]
pub struct FrameReader {
    buf: Vec<u8>,
}

impl FrameReader {
    /// A new, empty reader.
    pub fn new() -> Self {
        FrameReader { buf: Vec::new() }
    }

    /// Feed freshly-received bytes into the reader.
    ///
    /// Returns an error before appending if the buffered stream would exceed
    /// [`MAX_STREAM_BUFFER_BYTES`]. Callers should drain complete frames with
    /// [`FrameReader::next_frame`] before pushing more data.
    pub fn push(&mut self, bytes: &[u8]) -> Result<()> {
        let new_len =
            self.buf
                .len()
                .checked_add(bytes.len())
                .ok_or(BearerError::FrameTooLarge {
                    max: MAX_STREAM_BUFFER_BYTES,
                    got: usize::MAX,
                })?;
        if new_len > MAX_STREAM_BUFFER_BYTES {
            return Err(BearerError::FrameTooLarge {
                max: MAX_STREAM_BUFFER_BYTES,
                got: new_len,
            });
        }
        self.buf.extend_from_slice(bytes);
        Ok(())
    }

    /// Pop the next complete frame, if one has fully arrived.
    ///
    /// Returns `Ok(None)` when more bytes are needed, and an error if a frame
    /// header declares a length beyond [`MAX_FRAME_BYTES`].
    pub fn next_frame(&mut self) -> Result<Option<Vec<u8>>> {
        if self.buf.len() < 4 {
            return Ok(None);
        }
        let len = u32::from_be_bytes([self.buf[0], self.buf[1], self.buf[2], self.buf[3]]) as usize;
        if len > MAX_FRAME_BYTES {
            return Err(BearerError::FrameTooLarge {
                max: MAX_FRAME_BYTES,
                got: len,
            });
        }
        if self.buf.len() < 4 + len {
            return Ok(None);
        }
        let frame = self.buf[4..4 + len].to_vec();
        self.buf.drain(0..4 + len);
        Ok(Some(frame))
    }
}
