package org.mininet.app

import java.util.UUID

/**
 * The GATT profile shared by [BlePeripheralRadio] and [BleCentralRadio]:
 * one service, one write-only characteristic for central-to-peripheral
 * chunks, one notify-only characteristic for peripheral-to-central chunks.
 * Both classes only ever move opaque, already-chunked byte arrays produced
 * by `mini_bearer::ble::chunk_frame` across these -- nothing here parses a
 * chunk header or knows what a frame is; that stays entirely on the Rust
 * side of the UniFFI boundary ([BleBearerHandle]).
 *
 * The UUIDs below are this app's own profile identifiers, not a registered
 * Bluetooth SIG assigned number -- ordinary practice for an app-specific
 * GATT service (the same pattern as, e.g., the well-known Nordic UART
 * Service), fine to hardcode since they carry no secret.
 */
internal val MININET_BLE_SERVICE_UUID: UUID = UUID.fromString("8f4a2c2e-8b8f-4d8e-8c8a-2f6a6d696e69")

/** Central writes chunks here; the peripheral's GATT server receives them. */
internal val MININET_BLE_RX_CHARACTERISTIC_UUID: UUID = UUID.fromString("8f4a2c2f-8b8f-4d8e-8c8a-2f6a6d696e69")

/** Peripheral notifies chunks here; the central subscribes to receive them. */
internal val MININET_BLE_TX_CHARACTERISTIC_UUID: UUID = UUID.fromString("8f4a2c30-8b8f-4d8e-8c8a-2f6a6d696e69")

/** The standard Bluetooth SIG Client Characteristic Configuration descriptor. */
internal val CLIENT_CHARACTERISTIC_CONFIG_UUID: UUID = UUID.fromString("00002902-0000-1000-8000-00805f9b34fb")
