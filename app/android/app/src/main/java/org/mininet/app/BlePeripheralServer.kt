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
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.CountDownLatch
import java.util.concurrent.LinkedBlockingQueue
import java.util.concurrent.Semaphore
import java.util.concurrent.TimeUnit
import org.mininet.core.BleRadio
import org.mininet.core.BleRadioException

/**
 * The peripheral (GATT server / advertiser) half of the real Android
 * `BleRadio` (issue #201 Android beta slice 5; D-0374/D-0375 built the
 * Rust-side and UniFFI-boundary half this class drives; D-0502 shipped the
 * first, single-connection version of this class; this is its multi-central
 * redesign for `docs/design/ble-mesh-relay.md`'s mesh, closing the founder's
 * 2026-09-11 direction that devices form a real network over BLE, not just
 * pair one-to-one).
 *
 * One [BluetoothGattServer] already natively supports many simultaneously
 * connected centrals -- every server callback already carries the
 * [BluetoothDevice] it is about -- so this class tracks one [LinkState] per
 * connected central and, once each central has enabled notifications (ready
 * to receive), hands the caller a fresh [BleRadio] scoped to exactly that
 * one central via [start]'s `onLinkReady` callback. Each such [BleRadio]
 * becomes one edge a caller (`mini_mesh::MeshNode`, once wired through
 * `mini-ffi`) adds to its mesh -- this class knows nothing about meshing,
 * relaying, or how many hops away anyone is, only how to hold many
 * concurrent GATT-server links at once and expose each as a chunk-level
 * [BleRadio].
 *
 * **Serialization limit, stated plainly.** All connected centrals share one
 * GATT characteristic object (`txCharacteristic`); Android's pre-API-33
 * notify API reads its outgoing value from that shared object rather than
 * taking bytes as a parameter, and real devices are documented to sometimes
 * read that value at actual transmission time rather than at the
 * `notifyCharacteristicChanged` call itself -- so [sendGate] serializes
 * every [PeripheralLinkRadio.writeChunk] call, across every connected
 * central, for the *entire* round trip through [BluetoothGattServerCallback.onNotificationSent]:
 * two centrals cannot be notified concurrently, only in turn, and a second
 * send never starts until the previous one is actually acknowledged. This
 * is correct (no value ever gets read by the wrong recipient) but caps
 * peripheral-role throughput as more centrals connect; the real fix is
 * Android 13's four-argument `notifyCharacteristicChanged` overload, which
 * takes the value explicitly instead of reading shared characteristic
 * state -- left for later so this class supports the full `minSdk 26` range
 * uniformly rather than branching on API level for a first cut.
 *
 * **Honest limit**, unchanged from the single-connection version this
 * replaces: written without a JDK/Android SDK or BLE hardware in this
 * development environment, so this has never compiled or run. Android CI's
 * `assembleDebug` is the first real compile check; a real multi-device BLE
 * mesh test is the only thing that can prove this correct end to end.
 */
@SuppressLint("MissingPermission")
class BlePeripheralServer(context: Context) : BluetoothGattServerCallback() {
    private val appContext = context.applicationContext
    private val bluetoothManager =
        appContext.getSystemService(Context.BLUETOOTH_SERVICE) as BluetoothManager
    private val adapter = bluetoothManager.adapter

    /** Keyed by [BluetoothDevice.getAddress] -- one entry per connected central. */
    private val links = ConcurrentHashMap<String, LinkState>()

    /** Held for a whole send round trip; see the class doc's serialization limit. */
    private val sendGate = Semaphore(1)

    /**
     * Guards [pendingNotifyAck]/[pendingNotifyStatus] only -- never held
     * across a blocking wait, unlike [sendGate], so
     * [BluetoothGattServerCallback.onNotificationSent] (a different thread)
     * can always acquire it to signal completion without risking a
     * deadlock against a [writeChunk][PeripheralLinkRadio.writeChunk] call
     * that is itself waiting on that same signal.
     */
    private val ackLock = Object()

    private var gattServer: BluetoothGattServer? = null
    private var advertiser: BluetoothLeAdvertiser? = null
    private var txCharacteristic: BluetoothGattCharacteristic? = null

    @Volatile
    private var onLinkReady: ((BleRadio) -> Unit)? = null

    private class LinkState(val device: BluetoothDevice) {
        val incoming = LinkedBlockingQueue<ByteArray>()
        var notificationsEnabled = false
        var readyDelivered = false
    }

    /**
     * Opens the GATT server, adds the Mininet bearer service, and starts
     * advertising it. Returns once advertising has actually started (or
     * failed) -- unlike the single-connection version this replaces, this
     * does **not** block waiting for any particular central: any number of
     * centrals may connect over this server's lifetime, each announced
     * through [onLinkReady] the moment its notifications are enabled (ready
     * to both send and receive). Returns `false` if the server or
     * advertising could not start; the caller should [close] rather than
     * retry an already-partially-started server.
     */
    fun start(onLinkReady: (BleRadio) -> Unit, advertiseTimeoutMs: Long = DEFAULT_ADVERTISE_TIMEOUT_MS): Boolean {
        this.onLinkReady = onLinkReady

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
        advertiseStarted.await(advertiseTimeoutMs, TimeUnit.MILLISECONDS)
        if (!advertiseOk) {
            close()
            return false
        }
        return true
    }

    /** How many centrals are currently connected (not all necessarily ready yet). */
    fun connectedCount(): Int = links.size

    /** Stops advertising and releases the GATT server. Safe to call more than once. */
    fun close() {
        runCatching { advertiser?.stopAdvertising(object : AdvertiseCallback() {}) }
        runCatching { gattServer?.close() }
        gattServer = null
        advertiser = null
        links.clear()
    }

    override fun onConnectionStateChange(device: BluetoothDevice, status: Int, newState: Int) {
        when (newState) {
            BluetoothGatt.STATE_CONNECTED -> links[device.address] = LinkState(device)
            BluetoothGatt.STATE_DISCONNECTED -> links.remove(device.address)
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
        val state = links[device.address]
        if (characteristic.uuid == MININET_BLE_RX_CHARACTERISTIC_UUID && state != null) {
            state.incoming.put(value.copyOf())
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
        val state = links[device.address]
        if (descriptor.uuid == CLIENT_CHARACTERISTIC_CONFIG_UUID &&
            value.contentEquals(BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE) &&
            state != null
        ) {
            state.notificationsEnabled = true
            if (!state.readyDelivered) {
                state.readyDelivered = true
                onLinkReady?.invoke(PeripheralLinkRadio(state))
            }
        }
        if (responseNeeded) {
            gattServer?.sendResponse(device, requestId, BluetoothGatt.GATT_SUCCESS, offset, value)
        }
    }

    override fun onNotificationSent(device: BluetoothDevice, status: Int) {
        synchronized(ackLock) {
            pendingNotifyStatus = status
            pendingNotifyAck?.countDown()
        }
    }

    // At most one notify send is ever in flight across every connected
    // central (sendGate enforces that), so a single pending-ack pair
    // (rather than one per central) is always unambiguous. Only ever
    // touched while holding ackLock.
    private var pendingNotifyAck: CountDownLatch? = null
    private var pendingNotifyStatus = BluetoothGatt.GATT_SUCCESS

    /** One connected central's [BleRadio], sharing the outer server's GATT resources. */
    private inner class PeripheralLinkRadio(private val state: LinkState) : BleRadio {
        override fun writeChunk(chunk: List<UByte>) {
            val server = gattServer
                ?: throw BleRadioException.Failed("GATT server is closed")
            val characteristic = txCharacteristic
                ?: throw BleRadioException.Failed("service not started yet")
            if (links[state.device.address] == null) {
                throw BleRadioException.Failed("central disconnected")
            }

            try {
                sendGate.acquire()
            } catch (e: InterruptedException) {
                Thread.currentThread().interrupt()
                throw BleRadioException.Failed("interrupted while waiting to send: ${e.message}")
            }
            try {
                val latch = CountDownLatch(1)
                synchronized(ackLock) { pendingNotifyAck = latch }
                characteristic.setValue(chunk.toByteArray())
                val started = server.notifyCharacteristicChanged(state.device, characteristic, false)
                if (!started) {
                    synchronized(ackLock) { pendingNotifyAck = null }
                    throw BleRadioException.Failed("notifyCharacteristicChanged failed to start")
                }
                // Not holding ackLock here: onNotificationSent runs on a
                // different (Binder) thread and needs that same lock,
                // briefly, to signal this latch -- holding it across this
                // wait would deadlock the two threads against each other.
                // sendGate (held for this whole method) is what actually
                // keeps a second send from starting before this one settles.
                val acked = latch.await(NOTIFY_TIMEOUT_MS, TimeUnit.MILLISECONDS)
                val status = synchronized(ackLock) {
                    pendingNotifyAck = null
                    pendingNotifyStatus
                }
                if (!acked) {
                    throw BleRadioException.Failed(
                        "notification was not acknowledged within $NOTIFY_TIMEOUT_MS ms",
                    )
                }
                if (status != BluetoothGatt.GATT_SUCCESS) {
                    throw BleRadioException.Failed("notification failed with GATT status $status")
                }
            } finally {
                sendGate.release()
            }
        }

        override fun readChunk(): List<UByte> = state.incoming.take().toUByteList()

        override fun tryReadChunk(): List<UByte>? = state.incoming.poll()?.toUByteList()
    }

    companion object {
        private const val NOTIFY_TIMEOUT_MS = 10_000L
        private const val DEFAULT_ADVERTISE_TIMEOUT_MS = 10_000L
    }
}
