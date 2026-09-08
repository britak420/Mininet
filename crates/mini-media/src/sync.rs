//! Wiring this crate's want-lists ([`crate::missing_chunks`]/
//! [`crate::missing_superblock_chunks`]) into a real network fetch (roadmap
//! #35, huge-file handling; D-0419's own "Required follow-up": "wiring
//! `missing_superblock_chunks` into real `mini-sync` replication paths").
//!
//! Nothing here is a new wire protocol. [`mini_sync::request_retrieval`]/
//! [`mini_sync::serve_retrieval`] already implement a generic, already-
//! tested exact-object-retrieval exchange over any already-connected
//! [`mini_bearer::Bearer`]/[`mini_bearer::Channel`] — the same construction
//! `mini-search-federation-net` (D-0432) composes unmodified for its own,
//! unrelated object types. This module is the smallest thing that makes
//! that composition a *media* capability rather than something every caller
//! has to hand-assemble: [`pull_manifest`]/[`pull_superblock`] compute the
//! want-list and drive however many retrieval rounds it takes, and
//! [`serve_missing`] answers as many of those rounds as a peer sends before
//! disconnecting.
//!
//! ## Why a superblock needs *rounds*, not one call
//!
//! [`crate::missing_superblock_chunks`] can only report a part's *chunk*
//! ids once that part's own manifest has already arrived — until then, the
//! only thing to ask for is the manifest itself. A single-round fetch would
//! therefore stall on any superblock with more than one still-missing part.
//! [`pull_superblock`] repeats the want/fetch cycle until nothing is
//! missing — in practice two rounds (every missing part manifest, then
//! every chunk those manifests revealed) resolve any superblock. A peer
//! that cannot supply everything a round asks for is not silently retried:
//! `mini_sync::request_retrieval`'s own protocol requires a response to
//! cover every id requested, so that failure surfaces directly as
//! [`crate::MediaError::Sync`].
//!
//! ## Scope
//!
//! Transport-agnostic, exactly like [`mini_sync::request_retrieval`] itself:
//! these functions take an already-connected `Bearer`/`Channel`, the same
//! way `mini-search-federation-net`'s `pull_source`/`serve_source` do. They
//! do not dial a socket, and they are not a player: a caller still decides
//! when to fetch, from whom, and what to do with the result. No peer
//! selection, retry-across-peers, or multi-source-at-once policy — a failed
//! or incomplete round is the caller's to retry, against this or another
//! peer.

use mini_bearer::{Bearer, Channel};
use mini_objects::ObjectId;
use mini_store::{Backend, Store};
use mini_sync::{KelCache, RetrievalReport};

use crate::{missing_chunks, missing_superblock_chunks, Manifest, MediaError, Result, Superblock};

/// Bound on retrieval rounds [`pull_superblock`] will drive. Each round that
/// makes progress must reveal or fetch at least one previously-unresolved
/// object, so this is generous relative to [`crate::MAX_PARTS`] rather than
/// a value ever expected to bind in practice.
const MAX_SUPERBLOCK_ROUNDS: usize = crate::MAX_PARTS + 1;

/// Cumulative result of fetching every missing object for one manifest or
/// superblock — the per-round [`RetrievalReport`]s summed, plus how many
/// rounds it took.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MediaSyncReport {
    /// How many retrieval rounds were needed (1 for a flat manifest; a
    /// superblock may need more, one per newly-discovered part manifest).
    pub rounds: usize,
    /// Object ids requested across every round.
    pub requested: usize,
    /// Object ids the peer selected to serve across every round.
    pub selected: usize,
    /// Objects received over the wire, across every round.
    pub received: usize,
    /// Objects verified and accepted into the local store, across every
    /// round.
    pub accepted: usize,
}

impl MediaSyncReport {
    fn add_round(&mut self, report: &RetrievalReport) {
        self.rounds += 1;
        self.requested += report.requested;
        self.selected += report.selected;
        self.received += report.ingest.received;
        self.accepted += report.ingest.accepted;
    }
}

/// Fetch every chunk [`crate::missing_chunks`] reports absent for
/// `manifest`, from a peer, over an already-established `bearer`/`chan`.
/// A flat manifest's entire chunk list is known upfront, so this is always
/// exactly one retrieval round.
pub fn pull_manifest<B: Backend>(
    bearer: &mut dyn Bearer,
    chan: &mut Channel,
    store: &mut Store<B>,
    cache: &mut KelCache,
    manifest: &Manifest,
) -> Result<MediaSyncReport> {
    let mut report = MediaSyncReport::default();
    let missing = missing_chunks(store, manifest)?;
    if missing.is_empty() {
        return Ok(report);
    }
    let round = mini_sync::request_retrieval(bearer, chan, store, cache, &missing)?;
    report.add_round(&round);
    Ok(report)
}

/// Fetch every part manifest and chunk [`crate::missing_superblock_chunks`]
/// reports absent for `superblock`, from a peer, over an already-established
/// `bearer`/`chan` — repeating the want/fetch cycle as many times as newly
/// arrived part manifests reveal more missing chunks, until nothing remains
/// missing.
///
/// A round where the peer cannot supply everything just requested fails
/// outright with [`MediaError::Sync`] — `mini_sync::request_retrieval`'s own
/// protocol requires a response to cover every id it was asked for, so a
/// peer genuinely missing something is a fact reported by that failure, not
/// a reason this function retries on its own. Errors
/// [`MediaError::Incomplete`] if [`MAX_SUPERBLOCK_ROUNDS`] rounds still
/// leave something missing — a defensive bound, never expected to bind for
/// an honest superblock: fetching every currently-unknown part manifest
/// always fits in one round, and every chunk that manifest reveals fits in
/// the next, so two rounds resolve any superblock in practice.
pub fn pull_superblock<B: Backend>(
    bearer: &mut dyn Bearer,
    chan: &mut Channel,
    store: &mut Store<B>,
    cache: &mut KelCache,
    superblock: &Superblock,
) -> Result<MediaSyncReport> {
    let mut report = MediaSyncReport::default();
    for _ in 0..MAX_SUPERBLOCK_ROUNDS {
        let missing = missing_superblock_chunks(store, superblock)?;
        if missing.is_empty() {
            return Ok(report);
        }
        let round = mini_sync::request_retrieval(bearer, chan, store, cache, &missing)?;
        report.add_round(&round);
    }
    Err(MediaError::Incomplete)
}

/// Serve as many retrieval rounds as a peer sends, over an already-
/// established `bearer`/`chan`, from `store` — the server side of
/// [`pull_manifest`]/[`pull_superblock`]. Answers every request with
/// whatever of the requested ids `store` actually holds (never more, never
/// fewer than it can); a request for an id this store does not have simply
/// goes unserved, exactly [`mini_sync::request_retrieval`]'s own contract.
///
/// Loops until the peer cleanly disconnects (it has everything it needs);
/// any other transport error still propagates.
pub fn serve_missing<B: Backend>(
    bearer: &mut dyn Bearer,
    chan: &mut Channel,
    store: &Store<B>,
) -> Result<usize> {
    let mut rounds = 0;
    loop {
        let requested = match mini_sync::receive_retrieval_request(bearer, chan) {
            Ok(ids) => ids,
            Err(mini_sync::SyncError::Bearer(mini_bearer::BearerError::Closed)) => {
                return Ok(rounds)
            }
            Err(e) => return Err(e.into()),
        };
        let to_serve: Vec<ObjectId> = requested
            .into_iter()
            .filter(|id| store.contains(id).unwrap_or(false))
            .collect();
        if to_serve.is_empty() {
            // Every requested id is genuinely absent here: nothing to
            // stream back, and `serve_retrieval` itself refuses an empty
            // selection outright.
            return Err(MediaError::Incomplete);
        }
        mini_sync::serve_retrieval(bearer, chan, store, &to_serve)?;
        rounds += 1;
    }
}

#[cfg(test)]
mod tests {
    use did_mini::{Capabilities, Controller};
    use mini_bearer::{pair, InProcessBearer, Initiator, Responder};
    use mini_store::MemoryBackend;

    use super::*;
    use crate::{assemble, assemble_superblock, publish_large_media, publish_media};

    fn human(seed: u8) -> (Controller, Controller) {
        let mut root = Controller::incept_single_from_seeds(&[seed; 32], &[seed + 1; 32]).unwrap();
        let device = Controller::incept_device_single_from_seeds(
            &root.did(),
            &[seed + 2; 32],
            &[seed + 3; 32],
        )
        .unwrap();
        root.delegate_device(&device.did(), Capabilities::primary())
            .unwrap();
        (root, device)
    }

    fn channels(a: &mut InProcessBearer, b: &mut InProcessBearer) -> (Channel, Channel) {
        let (init, hello1) = Initiator::start().unwrap();
        a.send(&hello1).unwrap();
        let got1 = b.recv().unwrap();
        let (chan_b, hello2) = Responder::respond(&got1).unwrap();
        b.send(&hello2).unwrap();
        let got2 = a.recv().unwrap();
        (init.finish(&got2).unwrap(), chan_b)
    }

    #[test]
    fn pull_manifest_fetches_exactly_what_was_missing_and_reassembles_byte_identical() {
        let (root, device) = human(1);
        let mut server_store: Store<MemoryBackend> = Store::new(MemoryBackend::new());
        let bytes = vec![7u8; (CHUNK_SIZE_FOR_TESTS * 3) + 12];
        let manifest = publish_media(
            &mut server_store,
            &root.did(),
            &device,
            "application/octet-stream",
            &bytes,
            1,
            0,
        )
        .unwrap();

        let mut client_store: Store<MemoryBackend> = Store::new(MemoryBackend::new());
        let mut client_cache = KelCache::new();
        client_cache.insert_verified(root.kel());
        client_cache.insert_verified(device.kel());

        let (mut client_bearer, mut server_bearer) = pair();
        let (mut client_chan, mut server_chan) = channels(&mut client_bearer, &mut server_bearer);

        let manifest_for_server = manifest.clone();
        let server = std::thread::spawn(move || {
            serve_missing(&mut server_bearer, &mut server_chan, &server_store).unwrap();
            let _ = manifest_for_server;
        });

        let report = pull_manifest(
            &mut client_bearer,
            &mut client_chan,
            &mut client_store,
            &mut client_cache,
            &manifest,
        )
        .unwrap();
        drop(client_bearer);
        server.join().unwrap();

        assert_eq!(report.rounds, 1);
        assert_eq!(report.accepted, manifest.chunks.len());
        assert!(missing_chunks(&client_store, &manifest).unwrap().is_empty());
        assert_eq!(assemble(&client_store, &manifest).unwrap(), bytes);
    }

    #[test]
    fn pulling_an_already_complete_manifest_does_nothing() {
        let (root, device) = human(2);
        let mut store: Store<MemoryBackend> = Store::new(MemoryBackend::new());
        let manifest =
            publish_media(&mut store, &root.did(), &device, "text/plain", b"hi", 1, 0).unwrap();
        let mut cache = KelCache::new();

        // A real, valid, established channel -- the point of this test is
        // that `pull_manifest` never has to use it, because the client
        // already has every chunk.
        let (mut client_bearer, mut server_bearer) = pair();
        let (mut chan, _server_chan) = channels(&mut client_bearer, &mut server_bearer);

        let report = pull_manifest(
            &mut client_bearer,
            &mut chan,
            &mut store,
            &mut cache,
            &manifest,
        )
        .unwrap();
        assert_eq!(report, MediaSyncReport::default());
    }

    #[test]
    fn pull_superblock_drives_multiple_rounds_to_reveal_later_parts_chunks() {
        let (root, device) = human(3);
        let mut server_store: Store<MemoryBackend> = Store::new(MemoryBackend::new());
        // Three chunks per part, three parts: the client cannot learn part
        // 2's or part 3's chunk ids until each part's own manifest arrives.
        let part_len = CHUNK_SIZE_FOR_TESTS * 2;
        let bytes = vec![9u8; part_len * 3];
        let superblock = publish_large_media(
            &mut server_store,
            &root.did(),
            &device,
            "application/octet-stream",
            &bytes,
            2,
            1,
            0,
        )
        .unwrap();
        assert_eq!(superblock.parts.len(), 3);

        let mut client_store: Store<MemoryBackend> = Store::new(MemoryBackend::new());
        let mut client_cache = KelCache::new();
        client_cache.insert_verified(root.kel());
        client_cache.insert_verified(device.kel());

        let (mut client_bearer, mut server_bearer) = pair();
        let (mut client_chan, mut server_chan) = channels(&mut client_bearer, &mut server_bearer);

        let server = std::thread::spawn(move || {
            serve_missing(&mut server_bearer, &mut server_chan, &server_store).unwrap();
        });

        let report = pull_superblock(
            &mut client_bearer,
            &mut client_chan,
            &mut client_store,
            &mut client_cache,
            &superblock,
        )
        .unwrap();
        drop(client_bearer);
        server.join().unwrap();

        assert!(
            report.rounds > 1,
            "expected multiple rounds, got {report:?}"
        );
        assert!(missing_superblock_chunks(&client_store, &superblock)
            .unwrap()
            .is_empty());
        assert_eq!(
            assemble_superblock(&client_store, &superblock).unwrap(),
            bytes
        );
    }

    #[test]
    fn pull_superblock_fails_against_a_peer_missing_a_part() {
        let (root, device) = human(4);
        let mut full_store: Store<MemoryBackend> = Store::new(MemoryBackend::new());
        let bytes = vec![1u8; CHUNK_SIZE_FOR_TESTS * 5];
        let superblock = publish_large_media(
            &mut full_store,
            &root.did(),
            &device,
            "application/octet-stream",
            &bytes,
            2,
            1,
            0,
        )
        .unwrap();

        // A server store missing the *last* part's manifest entirely: it can
        // serve the first part's manifest+chunks, but can never satisfy the
        // rest.
        let mut partial_server_store: Store<MemoryBackend> = Store::new(MemoryBackend::new());
        for part_id in &superblock.parts[..superblock.parts.len() - 1] {
            let manifest = crate::read_manifest(&full_store.get(part_id).unwrap()).unwrap();
            partial_server_store
                .insert(&full_store.get(part_id).unwrap())
                .unwrap();
            for chunk_id in &manifest.chunks {
                partial_server_store
                    .insert(&full_store.get(chunk_id).unwrap())
                    .unwrap();
            }
        }
        let superblock_obj = full_store.get(&superblock.id).unwrap();
        partial_server_store.insert(&superblock_obj).unwrap();

        let mut client_store: Store<MemoryBackend> = Store::new(MemoryBackend::new());
        let mut client_cache = KelCache::new();
        client_cache.insert_verified(root.kel());
        client_cache.insert_verified(device.kel());

        let (mut client_bearer, mut server_bearer) = pair();
        let (mut client_chan, mut server_chan) = channels(&mut client_bearer, &mut server_bearer);

        let server = std::thread::spawn(move || {
            // Ignore the error: `mini_sync::request_retrieval`'s protocol
            // requires a response to cover every id it was asked for, so
            // the client fails as soon as it asks for the part this server
            // does not have, without ever echoing a confirmation back --
            // this call unblocks only once the client drops its side.
            let _ = serve_missing(&mut server_bearer, &mut server_chan, &partial_server_store);
        });

        let result = pull_superblock(
            &mut client_bearer,
            &mut client_chan,
            &mut client_store,
            &mut client_cache,
            &superblock,
        );
        drop(client_bearer);
        let _ = server.join();

        assert!(
            matches!(result, Err(MediaError::Sync(_))),
            "expected a sync failure, got {result:?}"
        );
    }

    const CHUNK_SIZE_FOR_TESTS: usize = crate::CHUNK_SIZE;
}
