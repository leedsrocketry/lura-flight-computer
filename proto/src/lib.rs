//! Code shared between the rocket (`onboard`) and the ground station (`ground`):
//! message types, encoding/decoding, CRC.
//!
//! This crate must stay `no_std` so it can run on the ESP32. Anything that
//! needs the standard library belongs in `ground` instead.
#![no_std]

/// Version of the telemetry protocol. Bump this whenever the message format
/// changes, so the ground station can tell it is talking to old firmware.
pub const PROTOCOL_VERSION: u8 = 1;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Frame {
    pub header: Header,
    pub message: Message,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Header {
    /// will wrap around, incremented by one
    pub sequence: u16,
    /// time the frame was sent,
    /// may differ from reading time
    pub time_ms: u32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum Message {
    Imu {
        time_ms: u32,
        accel: [i16; 3],
        gyro: [i16; 3],
    },
    Baro {
        time_ms: u32,
        pressure_pa: u32,
        temp_cc: i16,
    },
    Gps {
        time_ms: u32,
        lat_e7: i32,
        lon_e7: i32,
        alt_m: i32,
        sats: u8,
    },
}
