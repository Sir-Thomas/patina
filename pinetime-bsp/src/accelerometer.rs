use bma425::{BMA425, PowerMode};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_nrf::{gpio::Input, twim::Twim};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::Delay;
use patina_hal::accelerometer::Accelerometer;

/// Accelerometer Controller
pub struct AccelerometerController {
    accelerometer: BMA425<I2cDevice<'static, NoopRawMutex, Twim<'static>>, Input<'static>>,
}

impl AccelerometerController {
    pub(crate) fn new(
        i2c: I2cDevice<'static, NoopRawMutex, Twim<'static>>,
        interrupt_pin: Input<'static>,
    ) -> Self {
        let accelerometer = BMA425::new(i2c, interrupt_pin);
        Self { accelerometer }
    }
}

impl Accelerometer for AccelerometerController {
    async fn init(&mut self) {
        self.accelerometer.init(&mut Delay).await.unwrap()
    }

    async fn start(&mut self) {
        self.accelerometer.init(&mut Delay).await.unwrap();
        self.accelerometer
            .set_power_mode(PowerMode::Performance)
            .await
            .unwrap();
        self.accelerometer.set_acc_conf().await.unwrap();
    }

    async fn stop(&mut self) {
        self.accelerometer
            .set_power_mode(PowerMode::Off)
            .await
            .unwrap();
    }

    async fn read(&mut self) -> (i16, i16, i16) {
        self.accelerometer.get_accel_data().await.unwrap()
    }
}
