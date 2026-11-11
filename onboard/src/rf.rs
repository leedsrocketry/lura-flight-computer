//! Radio link: turns `Message`s into framed bytes and hands them to a LoRa radio.
//!
//! The chip driver sits behind the [`Radio`] trait. The hardware is not
//! designed yet, so [`Sx1262Radio`] targets the common SX1262 (868 MHz EU)
//! through the `lora-phy` crate; [`LogRadio`] is a print-only stand-in.

use embassy_time::Instant;
use esp_println::println;
use proto::{Frame, Header, Message};

/// Carrier frequency: EU 868 MHz sub-band g1 (868.0-868.6 MHz, 1 % duty
/// cycle, 25 mW ERP). 868.1 MHz is the first standard LoRaWAN channel.
pub const FREQUENCY_HZ: u32 = 868_100_000;

/// Transmit power. 14 dBm is the EU default limit for the g1 sub-band.
pub const TX_POWER_DBM: i32 = 14;

/// Largest encoded frame we will send. The biggest `Message` is ~40 bytes
/// after postcard + COBS; LoRa itself allows 255.
pub const MAX_FRAME_LEN: usize = 64;

/// What the telemetry code needs from a radio chip.
pub trait Radio {
    type Error: core::fmt::Debug;

    /// Send one packet and return once it is on air.
    async fn transmit(&mut self, payload: &[u8]) -> Result<(), Self::Error>;
}

#[allow(dead_code)] // payloads are only read through Debug
#[derive(Debug)]
pub enum Error<E> {
    /// The frame did not fit or could not be serialised.
    Coding(postcard::Error),
    /// The radio driver failed.
    Radio(E),
}

pub struct RF<R: Radio> {
    radio: R,
    sequence: u16,
}

impl<R: Radio> RF<R> {
    pub fn new(radio: R) -> Self {
        RF { radio, sequence: 0 }
    }

    /// Stamp, encode (postcard + COBS) and transmit one message.
    pub async fn transmit_message(&mut self, message: Message) -> Result<(), Error<R::Error>> {
        self.sequence = self.sequence.wrapping_add(1);
        let frame = Frame {
            header: Header {
                sequence: self.sequence,
                time_ms: now_ms(),
            },
            message,
        };

        let mut buf = [0u8; MAX_FRAME_LEN];
        let encoded = postcard::to_slice_cobs(&frame, &mut buf).map_err(Error::Coding)?;
        self.radio.transmit(encoded).await.map_err(Error::Radio)
    }
}

/// Milliseconds since boot, deliberately wrapped to 32 bits (~49 days).
pub fn now_ms() -> u32 {
    let low_bits = Instant::now().as_millis() & u64::from(u32::MAX);
    u32::try_from(low_bits).unwrap_or(0)
}

/// Stand-in radio that prints instead of transmitting.
#[allow(dead_code)] // for bench testing without hardware
pub struct LogRadio;

impl Radio for LogRadio {
    type Error = core::convert::Infallible;

    async fn transmit(&mut self, payload: &[u8]) -> Result<(), Self::Error> {
        println!("radio(sim): {} bytes {:02x?}", payload.len(), payload);
        Ok(())
    }
}

// ---- SX1262 driver ---------------------------------------------------------

use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    Async,
    gpio::{Input, Output},
    spi::master::Spi,
};
use lora_phy::{
    LoRa,
    iv::GenericSx126xInterfaceVariant,
    mod_params::{Bandwidth, CodingRate, ModulationParams, PacketParams, RadioError, SpreadingFactor},
    sx126x::{self, Sx126x, Sx1262},
};

type SpiDev = ExclusiveDevice<Spi<'static, Async>, Output<'static>, embassy_time::Delay>;
type Iv = GenericSx126xInterfaceVariant<Output<'static>, Input<'static>>;

/// Build the SPI device (bus + chip-select) for the radio.
pub fn spi_device(bus: Spi<'static, Async>, cs: Output<'static>) -> Result<SpiDev, core::convert::Infallible> {
    ExclusiveDevice::new(bus, cs, embassy_time::Delay)
}

/// SX1262 over SPI. DIO2 drives the RF switch, as on most modules.
pub struct Sx1262Radio {
    lora: LoRa<Sx126x<SpiDev, Iv, Sx1262>, embassy_time::Delay>,
    modulation: ModulationParams,
    packet: PacketParams,
}

impl Sx1262Radio {
    pub async fn new(
        spi: SpiDev,
        reset: Output<'static>,
        dio1: Input<'static>,
        busy: Input<'static>,
    ) -> Result<Self, RadioError> {
        let iv = GenericSx126xInterfaceVariant::new(reset, dio1, busy, None, None)?;
        let config = sx126x::Config {
            chip: Sx1262,
            // Set to Some(..) if the module has a TCXO on DIO3 (e.g. Ebyte E22).
            tcxo_ctrl: None,
            use_dcdc: true,
            rx_boost: false,
        };
        // Private sync word: other LoRa networks (LoRaWAN) won't be decoded.
        let mut lora = LoRa::new(Sx126x::new(spi, iv, config), false, embassy_time::Delay).await?;
        let modulation =
            lora.create_modulation_params(SpreadingFactor::_7, Bandwidth::_125KHz, CodingRate::_4_5, FREQUENCY_HZ)?;
        let packet = lora.create_tx_packet_params(8, false, true, false, &modulation)?;
        Ok(Sx1262Radio { lora, modulation, packet })
    }
}

impl Radio for Sx1262Radio {
    type Error = RadioError;

    async fn transmit(&mut self, payload: &[u8]) -> Result<(), RadioError> {
        self.lora
            .prepare_for_tx(&self.modulation, &mut self.packet, TX_POWER_DBM, payload)
            .await?;
        self.lora.tx().await
    }
}
