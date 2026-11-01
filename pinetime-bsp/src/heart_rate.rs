use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_nrf::{gpio::Input, twim::Twim};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use hrs3300::Hrs3300;

/// Heartrate Sensor
pub struct HeartRateSensor {
    controller: Hrs3300<I2cDevice<'static, NoopRawMutex, Twim<'static>>>,
}

impl HeartRateSensor {
    /// Create a new Heart Rate Sensor
    pub async fn new(
        i2c_bus: I2cDevice<'static, NoopRawMutex, Twim<'static>>,
        _input_pin: Input<'static>,
    ) -> Self {
        let mut controller = Hrs3300::new(i2c_bus);
        controller.init().await.unwrap();
        Self { controller }
    }

    /// Enable the Sensor
    pub async fn start_sensor(&mut self) {
        self.controller.enable_hrs().await.unwrap();
        self.controller.enable_oscillator().await.unwrap();
    }

    /// Disable the Sensor
    pub async fn stop_sensor(&mut self) {
        self.controller.disable_oscillator().await.unwrap();
        self.controller.disable_hrs().await.unwrap();
    }

    /// Read Heart Rate Sensor Data
    pub async fn read_hrs(&mut self) -> u32 {
        let value = self.controller.read_hrs().await.unwrap();
        value
    }

    /// Read Ambient Light Sensor Data
    pub async fn read_als(&mut self) -> u32 {
        let value = self.controller.read_als().await.unwrap();
        value
    }
}
