use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_nrf::{
    gpio::{Input, Output},
    twim::Twim,
};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use hrs3300::Hrs3300;

/// Heartrate Sensor
pub struct HeartRateSensor {
    controller: Hrs3300<I2cDevice<'static, NoopRawMutex, Twim<'static>>>,
}

impl HeartRateSensor {
    pub async fn new(
        i2c_bus: I2cDevice<'static, NoopRawMutex, Twim<'static>>,
        input_pin: Input<'static>,
    ) -> Self {
        let mut controller = Hrs3300::new(i2c_bus);
        controller.init().unwrap();
        Self { controller }
    }

    async fn simple_measure(&mut self) -> u32 {
        self.controller.enable_hrs().unwrap();
        let value = self.controller.read_hrs().unwrap();
        self.controller.disable_hrs().unwrap();
        value
    }
}
