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
import java.util.concurrent.RejectedExecutionException
import java.util.concurrent.ScheduledFuture
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
 * (returns `false`) if BLE permissions are not already granted; requesting
 * them from the user is the calling `Activity`'s job, kept separate on
 * purpose (this class has no `Activity` reference and never should). No
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

    private fun runOnWorker(block: () -> Unit) {
        if (closed) return
        try {
            worker.execute(block)
        } catch (_: RejectedExecutionException) {
            // Lost the race with close() between the check above and this
            // call -- not an error, just already shutting down.
        }
    }

    /**
     * Starts serving centrals (advertising) and scanning for peripherals,
     * both feeding [mesh], and begins polling it on a fixed schedule so
     * relayed traffic actually keeps moving instead of sitting queued
     * until some other caller happens to poll -- without this, a device in
     * the middle of a relay chain would only ever forward a message the
     * moment something else of its own called [MeshHandle.poll].
     * [onMessage] is invoked (on the polling thread -- hop to your own
     * thread if you touch UI) for every newly delivered message; the
     * default no-op still keeps the mesh relaying, it just drops the
     * payload for a caller that has nothing to do with it yet.
     *
     * Returns `false` if either role could not start -- most commonly: BLE
     * permissions not granted (including a fresh install with none granted
     * yet, surfaced as a caught [SecurityException] rather than a crash),
     * or no Bluetooth adapter. The caller should [close] rather than
     * assume a partial start is safe to retry.
     */
    fun start(onMessage: (List<UByte>) -> Unit = {}): Boolean {
        // BlePeripheralServer.start() handles its own SecurityException
        // (a fresh install without BLUETOOTH_ADVERTISE/CONNECT granted)
        // and reports that the same way as any other failure: a clean
        // `false` with its own state already cleaned up.
        val peripheralOk = peripheralServer.start(onLinkReady = { radio ->
            runOnWorker { runCatching { mesh.addAcceptedLink(radio, DEFAULT_MTU) } }
        })
        if (!peripheralOk) {
            close()
            return false
        }
        if (!startScanning()) {
            close()
            return false
        }
        pollTask = pollExecutor.scheduleWithFixedDelay({
            // mesh.poll() has already drained and recorded every one of
            // these in the seen cache by the time this runs -- a later
            // poll() cannot recover them. Catching around the whole loop
            // would let one throwing onMessage silently discard every
            // later message in the same batch; catching per-message keeps
            // one bad payload from taking the rest down with it.
            runCatching { mesh.poll() }.getOrNull()?.forEach { message ->
                runCatching { onMessage(message.payload) }
            }
        }, POLL_INTERVAL_MS, POLL_INTERVAL_MS, TimeUnit.MILLISECONDS)
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
    }
}
