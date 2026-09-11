package org.mininet.app

import android.annotation.SuppressLint
import android.bluetooth.BluetoothDevice
import android.bluetooth.BluetoothManager
import android.bluetooth.le.ScanCallback
import android.bluetooth.le.ScanFilter
import android.bluetooth.le.ScanResult
import android.bluetooth.le.ScanSettings
import android.content.Context
import android.os.ParcelUuid
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.CountDownLatch
import java.util.concurrent.Executors
import java.util.concurrent.LinkedBlockingQueue
import java.util.concurrent.RejectedExecutionException
import java.util.concurrent.ScheduledFuture
import java.util.concurrent.ThreadPoolExecutor
import java.util.concurrent.TimeUnit
import org.mininet.core.MeshHandle

/**
 * Orchestrates one device's whole BLE mesh presence (`docs/design/
 * ble-mesh-relay.md`, D-0503): advertises and serves centrals through
 * [BlePeripheralServer] *and* scans for and connects to other advertising
 * devices through [BleCentralRadio], feeding every resulting link into one
 * shared [MeshHandle], polled on a fixed schedule so relayed traffic
 * actually keeps moving -- the founder's 2026-09-11 direction that nearby
 * devices form a real network over BLE alone, not just pair one-to-one.
 *
 * A device that is both advertising (peripheral) and scanning (central) at
 * once naturally forms mesh edges in both directions: it serves whichever
 * nearby devices connect to it, and it connects to whichever nearby devices
 * it discovers -- exactly the dual role every node in a real mesh needs.
 *
 * **What this class deliberately does not do.** No retry or reconnect for
 * a link that drops -- a caller wanting resilience restarts [start] itself.
 * No connection-count cap -- a real deployment needs one (BLE radios and
 * batteries have real limits `docs/gates/hardware-test-protocol.md` is the
 * place to actually measure, not guess at here). No de-duplication against
 * a peripheral link to the same device this central role is also connected
 * to -- a harmless redundant edge, not a correctness problem, since
 * `MeshHandle`'s dedup is content-based and does not care how many edges a
 * message arrives on. No permission *request* flow -- [start] fails closed
 * (reports `false` to `onStarted`) if BLE permissions are not already
 * granted; requesting them from the user is the calling `Activity`'s job,
 * kept separate on purpose (this class has no `Activity` reference and
 * never should). No
 * restart: [close] shuts its worker/poll executors down permanently (and
 * marks the instance closed so any callback already in flight backs off
 * rather than racing that shutdown), so a service instance is
 * construct-`start`-`close` once; a caller wanting to rejoin the mesh
 * later constructs a fresh [BleMeshService].
 *
 * **Honest limit**, unchanged from [BlePeripheralServer]/[BleCentralRadio]:
 * written without a JDK/Android SDK or BLE hardware in this development
 * environment, so this has never compiled or run. Android CI's
 * `assembleDebug` is the first real compile check; a real multi-device
 * mesh test is the only thing that can prove this correct end to end.
 */
@SuppressLint("MissingPermission")
class BleMeshService(context: Context) {
    private val appContext = context.applicationContext
    private val bluetoothManager =
        appContext.getSystemService(Context.BLUETOOTH_SERVICE) as BluetoothManager

    /** The shared mesh every link this service forms is added to. */
    val mesh = MeshHandle()

    private val peripheralServer = BlePeripheralServer(appContext)

    /** Keyed by [BluetoothDevice.getAddress] -- one entry per central-role connection attempted or made. */
    private val centralLinks = ConcurrentHashMap<String, BleCentralRadio>()
    private var scanCallback: ScanCallback? = null

    @Volatile
    private var scanFailed = false

    // Flipped first in close(), before anything is actually torn down, so
    // a callback already in flight (a scan result, a link-ready signal)
    // that loses the race checks this and backs off instead of calling
    // execute() on an executor that is being (or has been) shut down --
    // ExecutorService.execute after shutdown() throws
    // RejectedExecutionException, which would otherwise turn ordinary
    // teardown into a crash on whatever Bluetooth callback thread lost
    // that race.
    @Volatile
    private var closed = false

    // add*Link performs a blocking Channel handshake (see MeshHandle's own
    // docs) and central scan/connect is itself blocking -- both happen off
    // the caller's thread here so start() itself returns quickly.
    private val worker = Executors.newCachedThreadPool { runnable ->
        Thread(runnable, "mininet-ble-mesh").apply { isDaemon = true }
    }

    // Separate from `worker`: polling must run on a fixed, short schedule
    // and must never queue behind a long blocking handshake call sharing
    // the same pool would otherwise force it to.
    private val pollExecutor = Executors.newSingleThreadScheduledExecutor { runnable ->
        Thread(runnable, "mininet-ble-mesh-poll").apply { isDaemon = true }
    }
    private var pollTask: ScheduledFuture<*>? = null

    // Bounded, unlike `worker`: message delivery is driven by mesh.poll()
    // on a fixed schedule and can submit up to MAX_MESSAGES_PER_LINK_PER_
    // POLL deliveries per link, every POLL_INTERVAL_MS, indefinitely. If
    // the caller's onMessage is slow (disk/network I/O) or -- as its own
    // contract explicitly allows -- simply never returns, an unbounded
    // pool (`worker`'s newCachedThreadPool) would keep creating threads to
    // run every new delivery until the process exhausts memory or native
    // thread handles, exactly what moving delivery off the poll thread was
    // supposed to prevent becoming a *different* resource exhaustion bug.
    // A small fixed pool with a bounded queue gives delivery real
    // throughput without that unbounded growth; DiscardPolicy is the
    // explicit overload response once the queue is full: silently drop
    // the newest delivery rather than blocking the submitter (that would
    // stall mesh.poll() itself, reintroducing the exact hazard this
    // executor exists to avoid) or growing without bound. Content-
    // addressed dedup means nothing about the mesh's own correctness
    // depends on every delivery actually reaching onMessage; relay
    // traffic itself keeps moving via mesh.poll() regardless of this
    // executor's backlog.
    private val deliveryExecutor = ThreadPoolExecutor(
        1,
        DELIVERY_MAX_THREADS,
        DELIVERY_KEEP_ALIVE_SECONDS,
        TimeUnit.SECONDS,
        LinkedBlockingQueue(DELIVERY_QUEUE_CAPACITY),
        { runnable -> Thread(runnable, "mininet-ble-mesh-deliver").apply { isDaemon = true } },
        ThreadPoolExecutor.DiscardPolicy(),
    )

    private fun runOnWorker(block: () -> Unit) {
        if (closed) return
        try {
            worker.execute {
                // Rechecked here, not just by the caller above: close() can
                // run in the full gap between that check and this queued
                // task actually starting (worker.shutdown() lets already-
                // submitted tasks run to completion, it does not cancel
                // them), which would otherwise let e.g. a discovery task
                // call connectGatt or add a mesh link after close() has
                // already closed and cleared everything.
                if (closed) return@execute
                block()
            }
        } catch (_: RejectedExecutionException) {
            // Lost the race with close() between the check above and this
            // call -- not an error, just already shutting down.
        }
    }

    // No try/catch around execute() here: ThreadPoolExecutor.DiscardPolicy
    // never throws RejectedExecutionException (unlike the default
    // AbortPolicy) -- a full queue or an already-shut-down executor both
    // just silently decline the task, which is exactly the intended
    // degradation for best-effort message delivery.
    private fun runOnDelivery(block: () -> Unit) {
        if (closed) return
        deliveryExecutor.execute {
            if (closed) return@execute
            block()
        }
    }

    /**
     * Starts serving centrals (advertising) and scanning for peripherals,
     * both feeding [mesh], and begins polling it on a fixed schedule so
     * relayed traffic actually keeps moving instead of sitting queued
     * until some other caller happens to poll -- without this, a device in
     * the middle of a relay chain would only ever forward a message the
     * moment something else of its own called [MeshHandle.poll].
     * [onMessage] is invoked (on this class's own worker thread -- hop to
     * your own thread if you touch UI) for every newly delivered message;
     * the default no-op still keeps the mesh relaying, it just drops the
     * payload for a caller that has nothing to do with it yet.
     *
     * Returns immediately; [onStarted] (also invoked on the worker thread)
     * is called once with whether startup actually succeeded. This is
     * deliberately async rather than blocking the caller: both
     * [BlePeripheralServer.start] and [startScanning] block synchronously
     * for real time (advertising/service registration, then confirming the
     * scan actually started) -- doing that on whatever thread calls
     * [start] would risk an ANR if that thread is Android's main thread,
     * exactly the caller this class is built for. `false` most commonly
     * means: BLE permissions not granted (including a fresh install with
     * none granted yet, surfaced as a caught [SecurityException] rather
     * than a crash), or no Bluetooth adapter. The caller should [close]
     * rather than assume a partial start is safe to retry.
     */
    fun start(onMessage: (List<UByte>) -> Unit = {}, onStarted: (Boolean) -> Unit = {}) {
        if (closed) {
            onStarted(false)
            return
        }
        runOnWorker { onStarted(startBlocking(onMessage)) }
    }

    private fun startBlocking(onMessage: (List<UByte>) -> Unit): Boolean {
        // BlePeripheralServer.start() handles its own SecurityException
        // (a fresh install without BLUETOOTH_ADVERTISE/CONNECT granted)
        // and reports that the same way as any other failure: a clean
        // `false` with its own state already cleaned up.
        val peripheralOk = peripheralServer.start(onLinkReady = { radio, disconnect ->
            runOnWorker {
                runCatching { mesh.addAcceptedLink(radio, DEFAULT_MTU) }
                    .onFailure {
                        // The handshake never completed (no hello, a
                        // malformed one, or it timed out): without this,
                        // the central stays connected and readyDelivered
                        // stays true, so it occupies a GATT connection and
                        // a `links` entry forever with nothing left to
                        // retry the handshake.
                        disconnect()
                    }
            }
        })
        // Rechecked after each blocking step below, not just implicitly at
        // the top: a concurrent close() that ran while peripheralServer
        // .start()/startScanning() was blocking would otherwise go
        // unnoticed here, letting this call start (or leave running) a
        // scan or advertisement that close() already believes it tore
        // down. close() is safe to call again -- it stops whatever this
        // call has started so far (it can now see, e.g., a scanCallback
        // startScanning() only just set) and leaves nothing orphaned.
        if (!peripheralOk || closed) {
            close()
            return false
        }
        val scanningOk = startScanning()
        if (!scanningOk || closed) {
            close()
            return false
        }
        val scheduled = runCatching {
            pollTask = pollExecutor.scheduleWithFixedDelay({
                // mesh.poll() has already drained and recorded every one of
                // these in the seen cache by the time this runs -- a later
                // poll() cannot recover them. Catching around the whole loop
                // would let one throwing onMessage silently discard every
                // later message in the same batch; catching per-message keeps
                // one bad payload from taking the rest down with it.
                runCatching { mesh.poll() }.getOrNull()?.forEach { message ->
                    // Dispatched onto the bounded deliveryExecutor, not
                    // run inline on this single scheduled poll thread and
                    // not on the unbounded `worker` pool either:
                    // onMessage is arbitrary caller code that might block
                    // (disk/network I/O) or simply never return, and this
                    // is the only thread driving mesh.poll() -- if
                    // onMessage ran here directly, one slow or hung
                    // handler would silently stop all relay traffic, not
                    // just delivery of that one message. See
                    // deliveryExecutor's own doc for why it must be
                    // bounded rather than reusing `worker`.
                    runOnDelivery { runCatching { onMessage(message.payload) } }
                }
            }, POLL_INTERVAL_MS, POLL_INTERVAL_MS, TimeUnit.MILLISECONDS)
        }.isSuccess
        if (!scheduled || closed) {
            close()
            return false
        }
        return true
    }

    /**
     * Whether the last scan-start attempt is known to have failed
     * (`onScanFailed`, e.g. `SCAN_FAILED_APPLICATION_REGISTRATION_FAILED`
     * or `SCAN_FAILED_ALREADY_STARTED`). `startScan` itself returns
     * nothing, so this is the only way a caller can learn scanning is not
     * actually working -- checked once, right after starting, to catch a
     * fast/synchronous failure, but a later, slower failure only ever
     * updates this flag; nothing here currently reacts to it beyond that
     * (real reconnect/retry policy is named as follow-up in the class doc).
     */
    fun isScanHealthy(): Boolean = !scanFailed

    private fun startScanning(): Boolean {
        val leScanner = bluetoothManager.adapter?.bluetoothLeScanner ?: return false
        val filter = ScanFilter.Builder()
            .setServiceUuid(ParcelUuid(MININET_BLE_SERVICE_UUID))
            .build()
        val settings = ScanSettings.Builder()
            .setScanMode(ScanSettings.SCAN_MODE_LOW_LATENCY)
            .build()
        val startedOrFailed = CountDownLatch(1)
        val callback = object : ScanCallback() {
            override fun onScanResult(callbackType: Int, result: ScanResult) {
                onDeviceDiscovered(result.device)
            }

            override fun onScanFailed(errorCode: Int) {
                scanFailed = true
                startedOrFailed.countDown()
            }
        }
        scanCallback = callback
        try {
            leScanner.startScan(listOf(filter), settings, callback)
        } catch (_: SecurityException) {
            scanCallback = null
            return false
        }
        // A real failure (e.g. registration failure) is normally reported
        // promptly; a short bounded wait catches that case synchronously
        // without holding start() hostage to scanning's own full lifetime.
        startedOrFailed.await(SCAN_START_CHECK_MS, TimeUnit.MILLISECONDS)
        if (scanFailed) {
            runCatching { leScanner.stopScan(callback) }
            scanCallback = null
            return false
        }
        return true
    }

    private fun onDeviceDiscovered(device: BluetoothDevice) {
        if (closed) return
        // putIfAbsent is the atomic "claim this address" step: onScanResult
        // fires repeatedly for the same still-advertising device, and only
        // the caller that wins the race should ever start a connectGatt for
        // it.
        val radio = BleCentralRadio(appContext)
        if (centralLinks.putIfAbsent(device.address, radio) != null) {
            return
        }
        // Covers the case the two explicit cleanups below (failed connect,
        // failed handshake) don't: a link that connects, becomes a working
        // mesh link, and only *later* has its peer disconnect. Without
        // this, centralLinks retains that BleCentralRadio (and, before the
        // GATT-close fix in failAndReleaseAll, its GATT client too) for
        // the rest of this service's lifetime even though mini_mesh has
        // already pruned the dead link on the Rust side.
        radio.setOnFailed { centralLinks.remove(device.address, radio) }
        runOnWorker {
            // connectAndAwaitReady, not scanConnectAndAwaitReady: this
            // service already runs its own continuous scan above, so
            // reusing that discovery instead of letting BleCentralRadio
            // start a second, independent scan avoids both the redundant
            // radio use and the risk of it finding and connecting to a
            // different device than the one that triggered this call.
            val ready = runCatching {
                radio.connectAndAwaitReady(device, CONNECT_TIMEOUT_MS)
            }.getOrDefault(false)
            if (!ready) {
                runCatching { radio.close() }
                centralLinks.remove(device.address)
                return@runOnWorker
            }
            runCatching { mesh.addDialedLink(radio, DEFAULT_MTU) }
                .onFailure {
                    runCatching { radio.close() }
                    centralLinks.remove(device.address)
                }
        }
    }

    /** How many mesh links (either role) are currently held. */
    fun linkCount(): Int = mesh.linkCount().toInt()

    /** Stops advertising, scanning, polling, and every held connection. */
    fun close() {
        closed = true
        pollTask?.cancel(false)
        pollTask = null
        runCatching { scanCallback?.let { bluetoothManager.adapter?.bluetoothLeScanner?.stopScan(it) } }
        scanCallback = null
        peripheralServer.close()
        centralLinks.values.forEach { runCatching { it.close() } }
        centralLinks.clear()
        worker.shutdown()
        pollExecutor.shutdown()
        deliveryExecutor.shutdown()
    }

    companion object {
        // The chunk-level MTU BleBearerHandle/AndroidBleBearer chunk to --
        // conservative (works even at BLE's pre-negotiation default ATT
        // MTU of 23 bytes minus ATT header overhead), not the larger MTU a
        // real negotiated connection usually allows. Tuning this to the
        // actual negotiated MTU per link is real follow-up work, not
        // correctness-required: a smaller MTU only means more chunks.
        private const val DEFAULT_MTU: UInt = 20u
        private const val CONNECT_TIMEOUT_MS = 15_000L
        private const val SCAN_START_CHECK_MS = 2_000L

        // How often the mesh is drained/relayed. Short enough that a
        // multi-hop relay chain stays responsive, long enough not to spin;
        // real tuning against actual radio/battery behavior is
        // hardware-gate territory, not guessed at here.
        private const val POLL_INTERVAL_MS = 250L

        // deliveryExecutor's bounds -- see its own doc for why it must be
        // bounded at all. A couple of threads is enough for onMessage
        // calls to make real progress concurrently without competing with
        // `worker`'s handshake/connect threads for CPU; the queue capacity
        // is generous relative to one poll cycle's worst case
        // (MAX_MESSAGES_PER_LINK_PER_POLL per link, across however many
        // links are held) without being large enough to hide a
        // persistently stuck onMessage for long before DiscardPolicy
        // starts shedding load.
        private const val DELIVERY_MAX_THREADS = 2
        private const val DELIVERY_KEEP_ALIVE_SECONDS = 30L
        private const val DELIVERY_QUEUE_CAPACITY = 256
    }
}
