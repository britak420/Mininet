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
import java.util.concurrent.Executors
import org.mininet.core.MeshHandle

/**
 * Orchestrates one device's whole BLE mesh presence (`docs/design/
 * ble-mesh-relay.md`, D-0503): advertises and serves centrals through
 * [BlePeripheralServer] *and* scans for and connects to other advertising
 * devices through [BleCentralRadio], feeding every resulting link into one
 * shared [MeshHandle] so a message broadcast by any device reaches every
 * other device it is transitively connected to -- the founder's
 * 2026-09-11 direction that nearby devices form a real network over BLE
 * alone, not just pair one-to-one.
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
 * message arrives on. No permission request flow -- [start] fails closed
 * (returns `false`) if BLE permissions are not already granted; requesting
 * them from the user is the calling `Activity`'s job, kept separate on
 * purpose (this class has no `Activity` reference and never should).
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

    // add*Link performs a blocking Channel handshake (see MeshHandle's own
    // docs) and central scan/connect is itself blocking -- both happen off
    // the caller's thread here so start() itself returns quickly.
    private val worker = Executors.newCachedThreadPool { runnable ->
        Thread(runnable, "mininet-ble-mesh").apply { isDaemon = true }
    }

    /**
     * Starts serving centrals (advertising) and scanning for peripherals,
     * both feeding [mesh]. Returns `false` if either could not start (most
     * commonly: BLE permissions not granted, or no Bluetooth adapter) --
     * the caller should [close] rather than assume a partial start is safe
     * to retry.
     */
    fun start(): Boolean {
        val peripheralOk = peripheralServer.start(onLinkReady = { radio ->
            worker.execute {
                runCatching { mesh.addAcceptedLink(radio, DEFAULT_MTU) }
            }
        })
        if (!peripheralOk) {
            return false
        }
        return startScanning()
    }

    private fun startScanning(): Boolean {
        val leScanner = bluetoothManager.adapter?.bluetoothLeScanner ?: return false
        val filter = ScanFilter.Builder()
            .setServiceUuid(ParcelUuid(MININET_BLE_SERVICE_UUID))
            .build()
        val settings = ScanSettings.Builder()
            .setScanMode(ScanSettings.SCAN_MODE_LOW_LATENCY)
            .build()
        val callback = object : ScanCallback() {
            override fun onScanResult(callbackType: Int, result: ScanResult) {
                onDeviceDiscovered(result.device)
            }
        }
        scanCallback = callback
        leScanner.startScan(listOf(filter), settings, callback)
        return true
    }

    private fun onDeviceDiscovered(device: BluetoothDevice) {
        // putIfAbsent is the atomic "claim this address" step: onScanResult
        // fires repeatedly for the same still-advertising device, and only
        // the caller that wins the race should ever start a connectGatt for
        // it.
        val radio = BleCentralRadio(appContext)
        if (centralLinks.putIfAbsent(device.address, radio) != null) {
            return
        }
        worker.execute {
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
                return@execute
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

    /** Stops advertising, scanning, and every held connection. */
    fun close() {
        runCatching { scanCallback?.let { bluetoothManager.adapter?.bluetoothLeScanner?.stopScan(it) } }
        scanCallback = null
        peripheralServer.close()
        centralLinks.values.forEach { runCatching { it.close() } }
        centralLinks.clear()
        worker.shutdown()
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
    }
}
