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
 * (issue #201 Android beta slice 5). Pairs with [BlePeripheralRadio] on the
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

    private val incoming = LinkedBlockingQueue<ByteArray>()
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
        scanner.startScan(listOf(filter), settings, scanCallback)
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
        gatt = device.connectGatt(appContext, false, this, BluetoothDevice.TRANSPORT_LE)

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

    private fun failAndReleaseAll() {
        connectFailed = true
        connectedLatch.countDown()
        servicesReadyLatch.countDown()
        notificationsReadyLatch.countDown()
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

        g.setCharacteristicNotification(tx, true)
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
            incoming.put(characteristic.value?.copyOf() ?: ByteArray(0))
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

    override fun readChunk(): List<UByte> = incoming.take().toUByteList()

    override fun tryReadChunk(): List<UByte>? = incoming.poll()?.toUByteList()

    companion object {
        private const val WRITE_TIMEOUT_MS = 10_000L
    }
}
