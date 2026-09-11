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
    private var advertiseCallback: AdvertiseCallback? = null
    private var txCharacteristic: BluetoothGattCharacteristic? = null

    // The second parameter lets a caller whose handshake fails disconnect
    // exactly this central (cancelConnection) rather than leaving its GATT
    // connection and LinkState occupying a slot forever -- this class knows
    // the device/gattServer needed to do that; the caller (mini_mesh's
    // handshake, driven from BleMeshService) only ever sees the BleRadio.
    @Volatile
    private var onLinkReady: ((BleRadio, disconnect: () -> Unit) -> Unit)? = null

    // onServiceAdded fires once per start() call (a fresh GATT server and
    // service each time), so a fresh latch/flag pair per instance is
    // correct -- this class is not meant to be started twice.
    private val serviceAddedLatch = CountDownLatch(1)
    @Volatile
    private var serviceAddedOk = false

    private class LinkState(val device: BluetoothDevice) {
        // Bounded: an untrusted central can write repeatedly to the
        // write-without-response RX characteristic, and every value lands
        // here regardless of whether anything is draining it yet (e.g. a
        // handshake that never completes). An unbounded queue would let
        // that grow without limit; offer() (not put()) drops writes past
        // capacity instead of blocking the Binder callback thread.
        val incoming = LinkedBlockingQueue<ByteArray>(INCOMING_QUEUE_CAPACITY)
        var notificationsEnabled = false
        var readyDelivered = false

        // Set once, from onConnectionStateChange's disconnect branch, on
        // the same LinkState instance a live PeripheralLinkRadio already
        // holds a reference to -- removing the address from the outer
        // `links` map alone does not reach a radio that was already handed
        // out, so readChunk/tryReadChunk check this directly instead of
        // relying on map membership.
        @Volatile
        var disconnected = false
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
     *
     * [onLinkReady] receives the new central's [BleRadio] plus a
     * `disconnect` callback: call it if whatever the caller does with the
     * radio next (e.g. a `Channel` handshake) fails, so a central that gets
     * this far but never completes a real handshake does not keep
     * occupying a GATT connection and a `links` entry forever.
     */
    fun start(
        onLinkReady: (BleRadio, disconnect: () -> Unit) -> Unit,
        advertiseTimeoutMs: Long = DEFAULT_ADVERTISE_TIMEOUT_MS,
    ): Boolean {
        return try {
            startInternal(onLinkReady, advertiseTimeoutMs)
        } catch (_: SecurityException) {
            // A fresh install with BLUETOOTH_ADVERTISE/CONNECT not yet
            // granted reaches a protected call somewhere in the sequence
            // below; this class's own documented contract is a clean
            // `false`, not a crash, so clean up whatever partial state
            // exists (an opened GATT server, e.g.) and report failure the
            // same as any other missing-prerequisite case.
            close()
            false
        }
    }

    private fun startInternal(
        onLinkReady: (BleRadio, disconnect: () -> Unit) -> Unit,
        advertiseTimeoutMs: Long,
    ): Boolean {
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

        val started = System.currentTimeMillis()
        fun remaining(): Long = (advertiseTimeoutMs - (System.currentTimeMillis() - started)).coerceAtLeast(0)

        // addService is fire-and-forget-looking but is not actually
        // synchronous: registration completes (or fails) asynchronously
        // through onServiceAdded, below. A central that connects and tries
        // service discovery before that completes would fail discovery
        // even though this server looks "started" -- so wait for the real
        // completion signal before ever advertising the service exists.
        if (!server.addService(service)) {
            close()
            return false
        }
        val serviceReady = serviceAddedLatch.await(remaining(), TimeUnit.MILLISECONDS)
        if (!serviceReady || !serviceAddedOk) {
            close()
            return false
        }

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
        val callback = object : AdvertiseCallback() {
            override fun onStartSuccess(settingsInEffect: AdvertiseSettings?) {
                advertiseOk = true
                advertiseStarted.countDown()
            }

            override fun onStartFailure(errorCode: Int) {
                advertiseOk = false
                advertiseStarted.countDown()
            }
        }
        // Stored so close() can stop this exact callback instance -- Android
        // matches stopAdvertising's callback by identity, so passing a
        // freshly constructed one there is a silent no-op that leaves the
        // radio advertising after "shutdown".
        advertiseCallback = callback
        advertiserInstance.startAdvertising(settings, data, callback)
        advertiseStarted.await(remaining(), TimeUnit.MILLISECONDS)
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
        // Must be the exact same callback instance startAdvertising was
        // given -- Android identifies an in-flight advertisement by that
        // identity, so a different (even functionally identical) instance
        // here silently fails to stop anything.
        advertiseCallback?.let { callback -> runCatching { advertiser?.stopAdvertising(callback) } }
        advertiseCallback = null
        runCatching { gattServer?.close() }
        gattServer = null
        advertiser = null
        links.clear()
    }

    override fun onServiceAdded(status: Int, service: BluetoothGattService) {
        serviceAddedOk = status == BluetoothGatt.GATT_SUCCESS
        serviceAddedLatch.countDown()
    }

    override fun onConnectionStateChange(device: BluetoothDevice, status: Int, newState: Int) {
        when (newState) {
            BluetoothGatt.STATE_CONNECTED -> links[device.address] = LinkState(device)
            // Mark the removed LinkState itself, not just the map entry --
            // a PeripheralLinkRadio already handed out via onLinkReady
            // holds a direct reference to this exact instance and has no
            // other way to learn the central is gone.
            BluetoothGatt.STATE_DISCONNECTED -> links.remove(device.address)?.let { it.disconnected = true }
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
            // offer(), not put(): a full queue (nobody draining -- most
            // likely a handshake that stalled or failed) drops this write
            // and disconnects the peer instead of growing without bound or
            // blocking this Binder callback thread.
            if (state.incoming.offer(value.copyOf())) {
                if (responseNeeded) {
                    server?.sendResponse(device, requestId, BluetoothGatt.GATT_SUCCESS, offset, null)
                }
            } else {
                if (responseNeeded) {
                    server?.sendResponse(device, requestId, BluetoothGatt.GATT_FAILURE, offset, null)
                }
                runCatching { server?.cancelConnection(device) }
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
                val readyDevice = state.device
                onLinkReady?.invoke(PeripheralLinkRadio(state)) {
                    runCatching { gattServer?.cancelConnection(readyDevice) }
                }
            }
        }
        if (responseNeeded) {
            gattServer?.sendResponse(device, requestId, BluetoothGatt.GATT_SUCCESS, offset, value)
        }
    }

    override fun onNotificationSent(device: BluetoothDevice, status: Int) {
        synchronized(ackLock) {
            // sendGate ensures at most one send is *started* at a time, but
            // it does not cancel the underlying async notify when a send
            // times out from writeChunk's perspective: sendGate is released
            // either way, so a next send (possibly for a different central)
            // can already be in flight by the time this late callback for
            // the timed-out one arrives. Only complete the latch that is
            // actually still waiting on *this* device's notification --
            // never let a late callback for device A complete a pending
            // send that has since moved on to device B.
            if (pendingNotifyDevice?.address != device.address) return
            pendingNotifyStatus = status
            pendingNotifyAck?.countDown()
        }
    }

    // Only ever touched while holding ackLock. pendingNotifyDevice is what
    // makes a stale onNotificationSent callback (for a send this method has
    // already given up on and released sendGate for) identifiable and
    // ignorable instead of silently completing whichever central's send
    // happens to be pending now.
    private var pendingNotifyDevice: BluetoothDevice? = null
    private var pendingNotifyAck: CountDownLatch? = null
    private var pendingNotifyStatus = BluetoothGatt.GATT_SUCCESS

    /** One connected central's [BleRadio], sharing the outer server's GATT resources. */
    private inner class PeripheralLinkRadio(private val state: LinkState) : BleRadio {
        override fun writeChunk(chunk: List<UByte>) {
            val server = gattServer
                ?: throw BleRadioException.Failed("GATT server is closed")
            val characteristic = txCharacteristic
                ?: throw BleRadioException.Failed("service not started yet")
            // Identity-checked (===), not just non-null: if this central
            // disconnected and a *new* connection from the same address
            // was accepted before this stale PeripheralLinkRadio's write
            // runs, links[state.device.address] now points at that new
            // LinkState, not this one. A bare null-check would let this
            // radio "successfully" notifyCharacteristicChanged the device
            // with ciphertext sealed under the old, superseded channel's
            // keys -- Android addresses that call by BluetoothDevice, so it
            // would reach the new connection and could corrupt its
            // handshake, not just silently fail.
            if (links[state.device.address] !== state) {
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
                synchronized(ackLock) {
                    pendingNotifyDevice = state.device
                    pendingNotifyAck = latch
                }
                characteristic.setValue(chunk.toByteArray())
                val started = server.notifyCharacteristicChanged(state.device, characteristic, false)
                if (!started) {
                    synchronized(ackLock) {
                        pendingNotifyAck = null
                        pendingNotifyDevice = null
                    }
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
                    pendingNotifyDevice = null
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

        // Bounded, not take(): an unbounded block here is exactly how a
        // central that completes the GATT connection/notification dance
        // but never actually sends a Channel hello would pin this thread
        // (and, via mesh.addAcceptedLink's own blocking handshake read,
        // one of BleMeshService's worker threads) forever -- an untrusted
        // nearby device could exhaust the pool that way. Bounding is safe
        // for this class's actual usage: after the one-shot handshake
        // read, mini_mesh::MeshNode only ever calls try_recv (never
        // blocking recv/readChunk again), so no legitimate caller needs
        // readChunk to block past a generous timeout.
        //
        // The timeout is ONE deadline across every call, not reset per
        // call: AndroidBleBearer::recv (the Rust side driving this during
        // the handshake) calls readChunk() repeatedly to reassemble a
        // single multi-chunk frame, and a fresh READ_TIMEOUT_MS on every
        // individual chunk would let an untrusted central declare a huge
        // chunk_count and send one valid-looking chunk just under the
        // timeout apart, pinning this thread (and, transitively, one of
        // BleMeshService's worker threads) far longer than the timeout is
        // meant to bound -- up to READ_TIMEOUT_MS times the chunk count.
        private var readDeadlineNanos: Long? = null

        override fun readChunk(): List<UByte> {
            val deadline = readDeadlineNanos ?: (System.nanoTime() + READ_TIMEOUT_MS * 1_000_000L).also {
                readDeadlineNanos = it
            }
            val remainingMs = ((deadline - System.nanoTime()) / 1_000_000L).coerceAtLeast(0L)
            val chunk = try {
                state.incoming.poll(remainingMs, TimeUnit.MILLISECONDS)
            } catch (e: InterruptedException) {
                Thread.currentThread().interrupt()
                throw BleRadioException.Failed("interrupted while waiting for a chunk: ${e.message}")
            }
            if (chunk != null) return chunk.toUByteList()
            if (state.disconnected) throw BleRadioException.Failed("central disconnected")
            throw BleRadioException.Failed("no chunk received within $READ_TIMEOUT_MS ms of the first")
        }

        // Buffered chunks are drained first regardless of disconnect state
        // (a central can disconnect right after its last legitimate write,
        // and that write is still real data); only an empty queue on an
        // already-disconnected link reports failure. This is the signal
        // mini_mesh::MeshNode::poll actually depends on to prune a dead
        // link -- it only ever calls the non-blocking try_recv path
        // (this method), never the blocking readChunk, once a link is
        // already established.
        override fun tryReadChunk(): List<UByte>? {
            val chunk = state.incoming.poll()
            if (chunk != null) return chunk.toUByteList()
            if (state.disconnected) throw BleRadioException.Failed("central disconnected")
            return null
        }

        // Called from Rust whenever the bearer wrapping this radio is
        // dropped for any reason -- including mini_mesh::MeshNode pruning
        // this link after a protocol or send failure, not only the
        // explicit `disconnect` closure onLinkReady's caller gets for a
        // failed handshake (see that callback's own doc comment). Same
        // action, `cancelConnection`: onConnectionStateChange's
        // STATE_DISCONNECTED branch removes this central's `LinkState`
        // from `links` and marks it disconnected once the platform
        // confirms the teardown, exactly as it already does for every
        // other disconnect path.
        //
        // Identity-checked (===), same reasoning as writeChunk above: if
        // this central already disconnected and reconnected with the same
        // address before this stale radio's drop got around to calling
        // disconnect(), links[state.device.address] now points at the
        // *new* LinkState, not this one. cancelConnection is addressed to
        // the BluetoothDevice, not to a specific LinkState, so an
        // unguarded call here would tear down the replacement connection
        // instead of doing nothing to an already-gone stale one.
        override fun disconnect() {
            if (links[state.device.address] !== state) {
                return
            }
            runCatching { gattServer?.cancelConnection(state.device) }
        }
    }

    companion object {
        private const val NOTIFY_TIMEOUT_MS = 10_000L
        private const val READ_TIMEOUT_MS = 30_000L
        private const val DEFAULT_ADVERTISE_TIMEOUT_MS = 10_000L

        // Generous relative to a real chunk (bounded by the negotiated ATT
        // MTU, typically well under 512 bytes) -- large enough that normal
        // mesh traffic never hits it, small enough that a peer that never
        // lets anything drain this queue (a stalled/failed handshake, e.g.)
        // is disconnected long before it costs meaningful memory.
        private const val INCOMING_QUEUE_CAPACITY = 4096
    }
}
