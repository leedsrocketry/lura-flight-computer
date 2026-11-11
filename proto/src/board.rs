#[derive(Clone, Debug)]
pub struct Board<'a> {
    pub pyro_arm: Option<GPIOPin>,
    pub outputs: &'a [Output],
    pub buses: &'a [BusInterface],
    pub sensors: Sensors,
    pub radios: &'a [Radio],
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct GPIOPin(pub u8);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct OutputId(pub u8);

#[derive(Clone, Debug)]
pub enum Output {
    Pyro {
        pin: GPIOPin,
        sense: Option<PyroSense>,
    },
    Level {
        pin: GPIOPin,
    },
    Mock {
        id: u8,
    },
}

#[derive(Clone, Debug)]
pub struct PyroSense {
    pub pin: GPIOPin,
    pub mode: SenseMode,
}

#[derive(Clone, Copy, Debug)]
pub enum SenseMode {
    Digital,
    ADC,
}

#[derive(Clone, Copy, Debug)]
pub enum I2cPeripheral {
    I2C0,
    I2C1,
}
#[derive(Clone, Copy, Debug)]
pub enum SpiPeripheral {
    SPI2,
    SPI3,
}
#[derive(Clone, Copy, Debug)]
pub enum UartPeripheral {
    UART0,
    UART1,
    UART2,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct BusInterfaceId(pub u8);

#[derive(Clone, Debug)]
pub enum BusInterface {
    I2c {
        peripheral: I2cPeripheral,
        sda: GPIOPin,
        scl: GPIOPin,
        frequency_hz: u32,
    },
    Spi {
        peripheral: SpiPeripheral,
        sclk: GPIOPin,
        mosi: GPIOPin,
        miso: GPIOPin,
        frequency_hz: u32,
    },
    Uart {
        peripheral: UartPeripheral,
        tx: GPIOPin,
        rx: GPIOPin,
        baud_rate_hz: u32,
    },
}

#[derive(Clone, Debug)]
pub enum BusBinding {
    I2c {
        bus: BusInterfaceId,
        address: u8,
    },
    Spi {
        bus: BusInterfaceId,
        cs: GPIOPin,
        interrupt: GPIOPin,
    },
    Uart(BusInterfaceId),
}

#[derive(Clone, Debug)]
pub struct Barometer {
    pub binding: BusBinding,
}

#[derive(Clone, Debug)]
pub struct IMU {
    pub binding: BusBinding,
}

#[derive(Clone, Debug)]
pub struct GPS {
    pub binding: BusBinding,
}

#[derive(Clone, Debug)]
pub struct Sensors {
    pub barometer: Barometer,
    pub imu: IMU,
    pub gps: GPS,
}

#[derive(Clone, Debug)]
pub struct Radio {
    pub binding: BusBinding,
    pub frequency_hz: u32,
    pub tx_power_dbm: u16,
}
