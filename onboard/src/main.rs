#![no_std]
#![no_main]

mod rf;

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Instant, Ticker, Timer};
use esp_backtrace as _;
use esp_hal::{
    gpio::{Input, InputConfig, Level, Output, OutputConfig},
    spi::master::{Config as SpiConfig, Spi},
    time::Rate,
    timer::timg::TimerGroup,
};
use esp_println::println;
use proto::Message;

use crate::rf::{RF, Sx1262Radio};

type Rf = RF<Sx1262Radio>;

esp_bootloader_esp_idf::esp_app_desc!();

/// A queue that passes readings from the sensor task to the telemetry task.
/// It holds up to 16 readings; if it is full, the sender waits.
static READINGS: Channel<CriticalSectionRawMutex, Message, 16> = Channel::new();

/// Pretend sensor: produces a reading 10 times per second.
#[embassy_executor::task]
async fn sensor_task() {
    let mut ticker = Ticker::every(Duration::from_hz(10));
    loop {
        let message = Message::Imu {
            time_ms: rf::now_ms(),
            accel: [1, 0, 1],
            gyro: [0, 1, 0],
        };
        READINGS.send(message).await;
        ticker.next().await;
    }
}

/// Pretend radio: takes readings out of the channel and "transmits" them.
/// The 50 ms delay stands in for a slow radio transmission. It does not
/// slow down the sensor task, because while this task waits, the others run.
#[embassy_executor::task]
async fn telemetry_task(radio: Sx1262Radio) {
    // The task owns the RF, so nothing needs to be borrowed for 'static.
    let mut rf = Rf::new(radio);
    loop {
        let message = READINGS.receive().await;
        if let Err(err) = rf.transmit_message(message).await {
            println!("TX failed: {:?}", err);
        }
    }
}

/// Bring up SPI, the control pins and the SX1262. Returns `None` (after
/// printing why) if any step fails.
#[allow(clippy::too_many_arguments)]
async fn init_radio(
    spi2: esp_hal::peripherals::SPI2<'static>,
    sck: esp_hal::peripherals::GPIO12<'static>,
    mosi: esp_hal::peripherals::GPIO11<'static>,
    miso: esp_hal::peripherals::GPIO13<'static>,
    nss: esp_hal::peripherals::GPIO10<'static>,
    reset: esp_hal::peripherals::GPIO6<'static>,
    busy: esp_hal::peripherals::GPIO5<'static>,
    dio1: esp_hal::peripherals::GPIO4<'static>,
) -> Option<Sx1262Radio> {
    let bus = match Spi::new(spi2, SpiConfig::default().with_frequency(Rate::from_mhz(8))) {
        Ok(bus) => bus
            .with_sck(sck)
            .with_mosi(mosi)
            .with_miso(miso)
            .into_async(),
        Err(err) => {
            println!("SPI config error: {:?}", err);
            return None;
        }
    };
    let nss = Output::new(nss, Level::High, OutputConfig::default());
    let reset = Output::new(reset, Level::High, OutputConfig::default());
    let busy = Input::new(busy, InputConfig::default());
    let dio1 = Input::new(dio1, InputConfig::default());

    let Ok(spi) = rf::spi_device(bus, nss);
    match Sx1262Radio::new(spi, reset, dio1, busy).await {
        Ok(radio) => Some(radio),
        Err(err) => {
            println!("SX1262 init error: {:?}", err);
            None
        }
    }
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Start the scheduler. It uses a hardware timer to know when to wake tasks.
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    println!("onboard: telemetry protocol v{}", proto::PROTOCOL_VERSION);

    match sensor_task() {
        Ok(token) => spawner.spawn(token),
        Err(err) => println!("could not start sensor_task: {:?}", err),
    }

    // Provisional SX1262 wiring (hardware not built yet; change freely).
    // Avoids strapping pins (0, 3, 45, 46) and flash/PSRAM pins (26-37).
    //   SCK=GPIO12  MOSI=GPIO11  MISO=GPIO13  NSS=GPIO10  (SPI2 default pins)
    //   RESET=GPIO6  BUSY=GPIO5  DIO1=GPIO4
    match init_radio(
        peripherals.SPI2,
        peripherals.GPIO12,
        peripherals.GPIO11,
        peripherals.GPIO13,
        peripherals.GPIO10,
        peripherals.GPIO6,
        peripherals.GPIO5,
        peripherals.GPIO4,
    )
    .await
    {
        Some(radio) => match telemetry_task(radio) {
            Ok(token) => spawner.spawn(token),
            Err(err) => println!("could not start telemetry_task: {:?}", err),
        },
        // Keep flying without telemetry rather than halting.
        None => println!("radio init failed; telemetry disabled"),
    }

    // main is a task too: use it as a once-per-second heartbeat.
    loop {
        println!("heartbeat: alive, uptime {} s", Instant::now().as_secs());
        Timer::after(Duration::from_secs(1)).await;
    }
}
