package org.mininet.app

import android.annotation.SuppressLint
import android.bluetooth.BluetoothDevice
import android.bluetooth.BluetoothGatt
import android.bluetooth.BluetoothGattCallback
import android.bluetooth.BluetoothGattCharacteristic
import android.bluetooth.BluetoothGattDescriptor
import android.bluetooth.BluetoothManager
import android.bluetooth.BluetoothProfile
import android.bluetooth.le.ScanCallback
import android.bluetooth.le.ScanFilter
import android.bluetooth.le.ScanResult
import android.bluetooth.le.ScanSettings
import android.content.Context
import android.os.ParcelUuid
import java.util.concurrent.CountDownLatch
import java.util.concurrent.LinkedBlockingQueue
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicReference
import org.mininet.core.BleRadio
import org.mininet.core.BleRadioException

/**
 * The central (GATT client / scanner) half of the real Android `BleRadio`
 * (issue #201 Android beta slice 5). Pairs with [BlePeripheralServer] on the
 * other phone -- see that class's doc comment for the shared GATT profile,
 * chunk-only responsibility, and honest limits, all of which apply
 * identically here.
 */
@SuppressLint("MissingPermission")
class BleCentralRadio(context: Context) : BluetoothGattCallback(), BleRadio {
    private val appContext = context.applicationContext
    private val bluetoothManager =
        appContext.getSystemService(Context.BLUETOOTH_SERVICE) as BluetoothManager
    private val adapter = bluetoothManager.adapter

    // Bounded, not unbounded: the peripheral this central connects to is an
    // untrusted nearby device that can send notifications before the
    // handshake completes or faster than polling drains them, and every
    // notification is copied here regardless of whether anything is
    // draining it yet. offer() (not put()) drops a notification past
    // capacity instead of growing without bound or blocking the Binder
    // callback thread -- the same discipline BlePeripheralServer.LinkState
    // already applies to its own incoming queue.
    private val incoming = LinkedBlockingQueue<ByteArray>(INCOMING_QUEUE_CAPACITY)
    private var gatt: BluetoothGatt? = null
    private var rxCharacteristic: BluetoothGattCharacteristic? = null

    private val connectedLatch = CountDownLatch(1)
    private val servicesReadyLatch = CountDownLatch(1)
    private val notificationsReadyLatch = CountDownLatch(1)

    @Volatile
    private var connectFailed = false

    private val writeLock = Object()
    private var pendingWriteAck: CountDownLatch? = null
    private var pendingWriteStatus = BluetoothGatt.GATT_SUCCESS

    /**
     * Scans for the Mininet bearer service, connects to the first
     * matching peripheral, discovers services, and enables notifications.
     * Blocks the calling thread until ready, or [timeoutMs] elapses --
     * at which point this returns `false` and the caller should [close]
     * rather than retry an already-started, possibly half-connected
     * session. A convenience wrapper around [connectAndAwaitReady] for a
     * caller with no scanner of its own; [BleMeshService] runs one
     * continuous scan for every nearby device instead and calls
     * [connectAndAwaitReady] directly per discovery, so it never starts a
     * second, redundant scan per connection attempt.
     */
    fun scanConnectAndAwaitReady(timeoutMs: Long): Boolean {
        val scanner = adapter?.bluetoothLeScanner ?: return false
        val start = System.currentTimeMillis()

        val scanFound = CountDownLatch(1)
        val foundDevice = AtomicReference<BluetoothDevice?>(null)
        val scanCallback = object : ScanCallback() {
            override fun onScanResult(callbackType: Int, result: ScanResult) {
                if (foundDevice.compareAndSet(null, result.device)) {
                    scanFound.countDown()
                }
            }

            override fun onScanFailed(errorCode: Int) {
                scanFound.countDown()
            }
        }
        val filter = ScanFilter.Builder()
            .setServiceUuid(ParcelUuid(MININET_BLE_SERVICE_UUID))
            .build()
        val settings = ScanSettings.Builder()
            .setScanMode(ScanSettings.SCAN_MODE_LOW_LATENCY)
            .build()
        try {
            scanner.startScan(listOf(filter), settings, scanCallback)
        } catch (_: SecurityException) {
            // A fresh install (or a user who revoked it) without
            // BLUETOOTH_SCAN granted yet -- the same failure
            // BleMeshService.startScanning() already catches for its own
            // continuous scan. No scan was ever started, so there is
            // nothing to stop; just report the clean failure this
            // function's contract already promises for "not ready."
            return false
        }
        scanFound.await(timeoutMs, TimeUnit.MILLISECONDS)
        runCatching { scanner.stopScan(scanCallback) }
        val device = foundDevice.get() ?: return false
        val remaining = (timeoutMs - (System.currentTimeMillis() - start)).coerceAtLeast(0)
        return connectAndAwaitReady(device, remaining)
    }

    /**
     * Connects to an already-discovered `device`, discovers services, and
     * enables notifications. Blocks the calling thread until ready, or
     * [timeoutMs] elapses -- at which point this returns `false` and the
     * caller should [close] rather than retry.
     */
    fun connectAndAwaitReady(device: BluetoothDevice, timeoutMs: Long): Boolean {
        val start = System.currentTimeMillis()
        try {
            gatt = device.connectGatt(appContext, false, this, BluetoothDevice.TRANSPORT_LE)
        } catch (_: SecurityException) {
            // Same missing-permission case scanConnectAndAwaitReady's own
            // startScan guards against, reachable here too since a caller
            // (BleMeshService) can invoke this directly with an
            // already-discovered device, skipping this class's own scan
            // step entirely. `gatt` is never assigned when this throws, so
            // there is no half-open GATT client for close() to clean up.
            return false
        }

        fun remaining(): Long = (timeoutMs - (System.currentTimeMillis() - start)).coerceAtLeast(0)
        if (!connectedLatch.await(remaining(), TimeUnit.MILLISECONDS) || connectFailed) return false
        if (!servicesReadyLatch.await(remaining(), TimeUnit.MILLISECONDS) || connectFailed) return false
        return notificationsReadyLatch.await(remaining(), TimeUnit.MILLISECONDS) && !connectFailed
    }

    /** Disconnects and releases the GATT client. Safe to call more than once. */
    fun close() {
        runCatching { gatt?.disconnect() }
        runCatching { gatt?.close() }
        gatt = null
    }

    // Set by the caller (BleMeshService) right after constructing this
    // radio, before ever connecting -- invoked once this radio has
    // terminally failed or disconnected (including well after a link was
    // already up and in use), so the caller can release whatever it holds
    // for this radio (BleMeshService.centralLinks' entry) without polling
    // or otherwise having to notice on its own that a central-role link
    // died. `@Volatile` since GATT callbacks and the caller's own thread
    // can both touch it.
    @Volatile
    private var onFailed: (() -> Unit)? = null

    fun setOnFailed(callback: () -> Unit) {
        onFailed = callback
    }

    private fun failAndReleaseAll() {
        connectFailed = true
        connectedLatch.countDown()
        servicesReadyLatch.countDown()
        notificationsReadyLatch.countDown()
        // Release the GATT client registration and callback immediately
        // rather than waiting for the whole BleMeshService to close --
        // without this, every central-role link that connects and later
        // disconnects (setup failure or a real peer going away well after
        // the link was up) leaks its GATT client for the rest of the
        // service's lifetime; a long-running mesh meeting many peers over
        // time can exhaust the platform's GATT connection budget purely
        // from ones already gone.
        runCatching { gatt?.close() }
        gatt = null
        onFailed?.invoke()
    }

    override fun onConnectionStateChange(g: BluetoothGatt, status: Int, newState: Int) {
        if (newState == BluetoothProfile.STATE_CONNECTED && status == BluetoothGatt.GATT_SUCCESS) {
            connectedLatch.countDown()
            g.discoverServices()
        } else if (newState == BluetoothProfile.STATE_DISCONNECTED) {
            failAndReleaseAll()
        }
    }

    override fun onServicesDiscovered(g: BluetoothGatt, status: Int) {
        if (status != BluetoothGatt.GATT_SUCCESS) {
            failAndReleaseAll()
            return
        }
        val service = g.getService(MININET_BLE_SERVICE_UUID)
        val rx = service?.getCharacteristic(MININET_BLE_RX_CHARACTERISTIC_UUID)
        val tx = service?.getCharacteristic(MININET_BLE_TX_CHARACTERISTIC_UUID)
        if (rx == null || tx == null) {
            failAndReleaseAll()
            return
        }
        rxCharacteristic = rx
        servicesReadyLatch.countDown()

        // setCharacteristicNotification only enables *local* delivery of
        // notifications this process already receives over the air; if it
        // returns false, writing the CCCD to ask the peripheral to *send*
        // them would still "succeed" while this side silently never
        // surfaces them, leaving readChunk waiting for input that already
        // arrived and was dropped. Fail the connection immediately instead.
        if (!g.setCharacteristicNotification(tx, true)) {
            connectFailed = true
            notificationsReadyLatch.countDown()
            return
        }
        val cccd = tx.getDescriptor(CLIENT_CHARACTERISTIC_CONFIG_UUID)
        if (cccd == null) {
            connectFailed = true
            notificationsReadyLatch.countDown()
            return
        }
        cccd.setValue(BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE)
        if (!g.writeDescriptor(cccd)) {
            connectFailed = true
            notificationsReadyLatch.countDown()
        }
    }

    override fun onDescriptorWrite(g: BluetoothGatt, descriptor: BluetoothGattDescriptor, status: Int) {
        if (descriptor.uuid == CLIENT_CHARACTERISTIC_CONFIG_UUID) {
            if (status != BluetoothGatt.GATT_SUCCESS) {
                connectFailed = true
            }
            notificationsReadyLatch.countDown()
        }
    }

    override fun onCharacteristicWrite(g: BluetoothGatt, characteristic: BluetoothGattCharacteristic, status: Int) {
        synchronized(writeLock) {
            pendingWriteStatus = status
            pendingWriteAck?.countDown()
        }
    }

    @Suppress("DEPRECATION", "OVERRIDE_DEPRECATION")
    override fun onCharacteristicChanged(g: BluetoothGatt, characteristic: BluetoothGattCharacteristic) {
        if (characteristic.uuid == MININET_BLE_TX_CHARACTERISTIC_UUID) {
            val value = characteristic.value?.copyOf() ?: ByteArray(0)
            if (!incoming.offer(value)) {
                // Nobody is draining fast enough -- most likely a peer that
                // completed the connection but never lets mesh polling
                // catch up, or is deliberately flooding notifications. Fail
                // this link the same way a genuine disconnect does instead
                // of growing memory without bound.
                connectFailed = true
                runCatching { g.disconnect() }
            }
        }
    }

    override fun writeChunk(chunk: List<UByte>) {
        val g = gatt ?: throw BleRadioException.Failed("not connected")
        val characteristic = rxCharacteristic
            ?: throw BleRadioException.Failed("service not discovered yet")

        val latch = CountDownLatch(1)
        synchronized(writeLock) { pendingWriteAck = latch }
        characteristic.setValue(chunk.toByteArray())
        characteristic.writeType = BluetoothGattCharacteristic.WRITE_TYPE_DEFAULT
        if (!g.writeCharacteristic(characteristic)) {
            synchronized(writeLock) { pendingWriteAck = null }
            throw BleRadioException.Failed("writeCharacteristic failed to start")
        }
        val acked = latch.await(WRITE_TIMEOUT_MS, TimeUnit.MILLISECONDS)
        val status = synchronized(writeLock) {
            pendingWriteAck = null
            pendingWriteStatus
        }
        if (!acked) {
            throw BleRadioException.Failed("characteristic write was not acknowledged within $WRITE_TIMEOUT_MS ms")
        }
        if (status != BluetoothGatt.GATT_SUCCESS) {
            throw BleRadioException.Failed("characteristic write failed with GATT status $status")
        }
    }

    // Bounded, not take() -- see BlePeripheralServer.PeripheralLinkRadio's
    // own readChunk for why an unbounded wait here is a real hazard (a
    // connected peer that never sends a Channel hello pins this thread
    // forever) and why bounding it is safe for this class's actual usage
    // (mini_mesh::MeshNode never calls the blocking recv/readChunk path
    // again after the one-shot handshake read).
    //
    // The timeout is ONE deadline across every call, not reset per call --
    // same reasoning as BlePeripheralServer.PeripheralLinkRadio.readChunk:
    // AndroidBleBearer::recv calls readChunk() repeatedly to reassemble one
    // multi-chunk handshake frame, so resetting the timeout on each
    // individual chunk would let an untrusted peripheral string this
    // thread along far past READ_TIMEOUT_MS by sending one chunk just
    // under the timeout apart.
    private var readDeadlineNanos: Long? = null

    override fun readChunk(): List<UByte> {
        val deadline = readDeadlineNanos ?: (System.nanoTime() + READ_TIMEOUT_MS * 1_000_000L).also {
            readDeadlineNanos = it
        }
        val remainingMs = ((deadline - System.nanoTime()) / 1_000_000L).coerceAtLeast(0L)
        val chunk = try {
            incoming.poll(remainingMs, TimeUnit.MILLISECONDS)
        } catch (e: InterruptedException) {
            Thread.currentThread().interrupt()
            throw BleRadioException.Failed("interrupted while waiting for a chunk: ${e.message}")
        }
        if (chunk != null) return chunk.toUByteList()
        if (connectFailed) throw BleRadioException.Failed("peripheral disconnected")
        throw BleRadioException.Failed("no chunk received within $READ_TIMEOUT_MS ms of the first")
    }

    // Buffered chunks are drained first regardless of failure state, same
    // reasoning as BlePeripheralServer.PeripheralLinkRadio.tryReadChunk --
    // only an empty queue on an already-failed link reports failure. This
    // is the signal mini_mesh::MeshNode::poll depends on to prune a dead
    // central-role link: it only ever calls the non-blocking try_recv path
    // once a link is established.
    override fun tryReadChunk(): List<UByte>? {
        val chunk = incoming.poll()
        if (chunk != null) return chunk.toUByteList()
        if (connectFailed) throw BleRadioException.Failed("peripheral disconnected")
        return null
    }

    // Called from Rust whenever the bearer wrapping this radio is dropped
    // for any reason -- including mini_mesh::MeshNode pruning this link
    // after a protocol or send failure, not only an explicit caller-driven
    // close. Reuses close() so both paths tear down the same GATT client
    // state identically.
    override fun disconnect() {
        close()
    }

    companion object {
        private const val WRITE_TIMEOUT_MS = 10_000L
        private const val READ_TIMEOUT_MS = 30_000L

        // Same bound and reasoning as BlePeripheralServer's
        // INCOMING_QUEUE_CAPACITY: generous relative to a real chunk,
        // small enough that a peer nobody is draining is disconnected
        // long before it costs meaningful memory.
        private const val INCOMING_QUEUE_CAPACITY = 4096
    }
}
