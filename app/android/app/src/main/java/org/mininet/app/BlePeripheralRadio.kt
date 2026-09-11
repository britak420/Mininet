package org.mininet.app

import android.annotation.SuppressLint
import android.bluetooth.BluetoothDevice
import android.bluetooth.BluetoothGatt
import android.bluetooth.BluetoothGattCharacteristic
import android.bluetooth.BluetoothGattDescriptor
import android.bluetooth.BluetoothGattServer
import android.bluetooth.BluetoothGattServerCallback
import android.bluetooth.BluetoothGattService
import android.bluetooth.BluetoothManager
import android.bluetooth.le.AdvertiseCallback
import android.bluetooth.le.AdvertiseData
import android.bluetooth.le.AdvertiseSettings
import android.bluetooth.le.BluetoothLeAdvertiser
import android.content.Context
import android.os.ParcelUuid
import java.util.concurrent.CountDownLatch
import java.util.concurrent.LinkedBlockingQueue
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicReference
import org.mininet.core.BleRadio
import org.mininet.core.BleRadioException

/**
 * The peripheral (GATT server / advertiser) half of the real Android
 * `BleRadio` (issue #201 Android beta slice 5; D-0374/D-0375 built the
 * Rust-side and UniFFI-boundary half this class finally drives). One phone
 * in a keystone pairing opens this role; the other phone opens
 * [BleCentralRadio] -- there is no in-band negotiation over which phone
 * takes which role, matching how `mini_bearer::{Initiator, Responder}`
 * already leave that choice to the caller.
 *
 * This class moves only opaque, already-chunked byte arrays: one GATT
 * characteristic-write request in, one notification out. All framing,
 * ordering, and reassembly happen entirely on the Rust side of the UniFFI
 * boundary (`BleBearerHandle` wrapping `mini_bearer::AndroidBleBearer`) --
 * this class does not know what a "frame" is, only what a "chunk" is.
 *
 * **Honest limit.** Written without a JDK/Android SDK available in this
 * development environment, so this has never actually compiled or run
 * anywhere. Android CI's `assembleDebug` is the first real compile check;
 * a real two-device BLE connection -- the exact remaining piece
 * `docs/BETA_STATUS.md` item 1 names -- is the only thing that can prove
 * this protocol implementation is correct end to end, not merely
 * structurally plausible against the documented GATT APIs. Not yet wired
 * into [MiniViewModel]'s pairing flow; that is separate, later work.
 */
@SuppressLint("MissingPermission")
class BlePeripheralRadio(context: Context) : BluetoothGattServerCallback(), BleRadio {
    private val appContext = context.applicationContext
    private val bluetoothManager =
        appContext.getSystemService(Context.BLUETOOTH_SERVICE) as BluetoothManager
    private val adapter = bluetoothManager.adapter

    private val incoming = LinkedBlockingQueue<ByteArray>()
    private val connectedDevice = AtomicReference<BluetoothDevice?>(null)
    private val notificationsEnabled = AtomicReference<CountDownLatch?>(CountDownLatch(1))

    private val notifyLock = Object()
    private var pendingNotifyAck: CountDownLatch? = null
    private var pendingNotifyStatus = BluetoothGatt.GATT_SUCCESS

    private var gattServer: BluetoothGattServer? = null
    private var advertiser: BluetoothLeAdvertiser? = null
    private var txCharacteristic: BluetoothGattCharacteristic? = null

    /**
     * Opens the GATT server, adds the Mininet bearer service, and starts
     * advertising it. Blocks the calling thread until a central both
     * connects and enables notifications on the TX characteristic (ready
     * to receive), or [timeoutMs] elapses -- at which point this returns
     * `false` and the caller should [close] and give up rather than retry
     * an already-started, possibly half-connected server.
     */
    fun startAndAwaitConnection(timeoutMs: Long): Boolean {
        val started = System.currentTimeMillis()
        val rx = BluetoothGattCharacteristic(
            MININET_BLE_RX_CHARACTERISTIC_UUID,
            BluetoothGattCharacteristic.PROPERTY_WRITE or BluetoothGattCharacteristic.PROPERTY_WRITE_NO_RESPONSE,
            BluetoothGattCharacteristic.PERMISSION_WRITE,
        )
        val tx = BluetoothGattCharacteristic(
            MININET_BLE_TX_CHARACTERISTIC_UUID,
            BluetoothGattCharacteristic.PROPERTY_NOTIFY,
            BluetoothGattCharacteristic.PERMISSION_READ,
        )
        val cccd = BluetoothGattDescriptor(
            CLIENT_CHARACTERISTIC_CONFIG_UUID,
            BluetoothGattDescriptor.PERMISSION_READ or BluetoothGattDescriptor.PERMISSION_WRITE,
        )
        tx.addDescriptor(cccd)
        txCharacteristic = tx

        val service = BluetoothGattService(MININET_BLE_SERVICE_UUID, BluetoothGattService.SERVICE_TYPE_PRIMARY)
        service.addCharacteristic(rx)
        service.addCharacteristic(tx)

        val server = bluetoothManager.openGattServer(appContext, this) ?: return false
        gattServer = server
        server.addService(service)

        val advertiserInstance = adapter?.bluetoothLeAdvertiser
        if (advertiserInstance == null) {
            close()
            return false
        }
        advertiser = advertiserInstance

        val settings = AdvertiseSettings.Builder()
            .setAdvertiseMode(AdvertiseSettings.ADVERTISE_MODE_LOW_LATENCY)
            .setTxPowerLevel(AdvertiseSettings.ADVERTISE_TX_POWER_HIGH)
            .setConnectable(true)
            .build()
        val data = AdvertiseData.Builder()
            .setIncludeDeviceName(false)
            .addServiceUuid(ParcelUuid(MININET_BLE_SERVICE_UUID))
            .build()

        val advertiseStarted = CountDownLatch(1)
        var advertiseOk = false
        val advertiseCallback = object : AdvertiseCallback() {
            override fun onStartSuccess(settingsInEffect: AdvertiseSettings?) {
                advertiseOk = true
                advertiseStarted.countDown()
            }

            override fun onStartFailure(errorCode: Int) {
                advertiseOk = false
                advertiseStarted.countDown()
            }
        }
        advertiserInstance.startAdvertising(settings, data, advertiseCallback)
        advertiseStarted.await(timeoutMs, TimeUnit.MILLISECONDS)
        if (!advertiseOk) {
            close()
            return false
        }

        val remaining = (timeoutMs - (System.currentTimeMillis() - started)).coerceAtLeast(0)
        val latch = notificationsEnabled.get()
        val ready = latch == null || latch.await(remaining, TimeUnit.MILLISECONDS)
        return ready && connectedDevice.get() != null
    }

    /** Stops advertising and releases the GATT server. Safe to call more than once. */
    fun close() {
        runCatching { advertiser?.stopAdvertising(object : AdvertiseCallback() {}) }
        runCatching { gattServer?.close() }
        gattServer = null
        advertiser = null
    }

    override fun onConnectionStateChange(device: BluetoothDevice, status: Int, newState: Int) {
        when (newState) {
            BluetoothGatt.STATE_CONNECTED -> connectedDevice.set(device)
            BluetoothGatt.STATE_DISCONNECTED -> {
                connectedDevice.compareAndSet(device, null)
            }
        }
    }

    override fun onCharacteristicWriteRequest(
        device: BluetoothDevice,
        requestId: Int,
        characteristic: BluetoothGattCharacteristic,
        preparedWrite: Boolean,
        responseNeeded: Boolean,
        offset: Int,
        value: ByteArray,
    ) {
        val server = gattServer
        if (characteristic.uuid == MININET_BLE_RX_CHARACTERISTIC_UUID) {
            incoming.put(value.copyOf())
            if (responseNeeded) {
                server?.sendResponse(device, requestId, BluetoothGatt.GATT_SUCCESS, offset, null)
            }
        } else if (responseNeeded) {
            server?.sendResponse(device, requestId, BluetoothGatt.GATT_FAILURE, offset, null)
        }
    }

    override fun onDescriptorWriteRequest(
        device: BluetoothDevice,
        requestId: Int,
        descriptor: BluetoothGattDescriptor,
        preparedWrite: Boolean,
        responseNeeded: Boolean,
        offset: Int,
        value: ByteArray,
    ) {
        if (descriptor.uuid == CLIENT_CHARACTERISTIC_CONFIG_UUID &&
            value.contentEquals(BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE)
        ) {
            notificationsEnabled.getAndSet(null)?.countDown()
        }
        if (responseNeeded) {
            gattServer?.sendResponse(device, requestId, BluetoothGatt.GATT_SUCCESS, offset, value)
        }
    }

    override fun onNotificationSent(device: BluetoothDevice, status: Int) {
        synchronized(notifyLock) {
            pendingNotifyStatus = status
            pendingNotifyAck?.countDown()
        }
    }

    override fun writeChunk(chunk: List<UByte>) {
        val device = connectedDevice.get()
            ?: throw BleRadioException.Failed("no central is connected")
        val server = gattServer
            ?: throw BleRadioException.Failed("GATT server is closed")
        val characteristic = txCharacteristic
            ?: throw BleRadioException.Failed("service not started yet")

        val latch = CountDownLatch(1)
        synchronized(notifyLock) { pendingNotifyAck = latch }
        characteristic.setValue(chunk.toByteArray())
        val started = server.notifyCharacteristicChanged(device, characteristic, false)
        if (!started) {
            synchronized(notifyLock) { pendingNotifyAck = null }
            throw BleRadioException.Failed("notifyCharacteristicChanged failed to start")
        }
        val acked = latch.await(NOTIFY_TIMEOUT_MS, TimeUnit.MILLISECONDS)
        val status = synchronized(notifyLock) {
            pendingNotifyAck = null
            pendingNotifyStatus
        }
        if (!acked) {
            throw BleRadioException.Failed("notification was not acknowledged within $NOTIFY_TIMEOUT_MS ms")
        }
        if (status != BluetoothGatt.GATT_SUCCESS) {
            throw BleRadioException.Failed("notification failed with GATT status $status")
        }
    }

    override fun readChunk(): List<UByte> = incoming.take().toUByteList()

    override fun tryReadChunk(): List<UByte>? = incoming.poll()?.toUByteList()

    companion object {
        private const val NOTIFY_TIMEOUT_MS = 10_000L
    }
}
