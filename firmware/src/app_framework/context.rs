use defmt::info;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::watch::DynAnonReceiver;
use embassy_time::{Delay, Duration};
use nrf_sdc::mpsl::Flash;
use pinetime_bsp::backlight::BacklightController;
use pinetime_bsp::vibrator::Vibrator;
use pinetime_bsp::{Accelerometer, BrightnessLevel, PowerMode};
use time::PlainDateTime;

use crate::app_framework::prelude::*;
use crate::signals::{ADJUST_TIME, BATTERY, CURRENT_TIME};

const VALID_BIT_ADDRESS: u32 = 0x7BFE8;
const VALID_BIT_VALUE: u32 = 1;

#[derive(Clone, Debug)]
pub enum Command {
    RotateScreen,
}

pub struct AppContext {
    accelerometer: Accelerometer,
    pub backlight: BacklightController,
    bluetooth_connected: bool,
    current_time_watcher: DynAnonReceiver<'static, PlainDateTime>,
    flash: &'static Mutex<NoopRawMutex, Flash<'static>>,
    pub screen_on: bool,
    vibrator: Vibrator,
    pub commands: Vec<Command, 8>,
}

impl AppContext {
    pub fn new(
        accelerometer: Accelerometer,
        backlight: BacklightController,
        flash: &'static Mutex<NoopRawMutex, Flash<'static>>,
        vibrator: Vibrator,
    ) -> Self {
        let current_time_watcher = CURRENT_TIME.dyn_anon_receiver();
        Self {
            accelerometer,
            backlight,
            bluetooth_connected: false,
            current_time_watcher,
            flash,
            screen_on: true,
            vibrator,
            commands: Vec::new(),
        }
    }

    pub fn display_is_off(&self) -> bool {
        !self.screen_on
    }

    pub fn time(&mut self) -> PlainDateTime {
        let current_time = self
            .current_time_watcher
            .try_get()
            .expect("There will always be a valid time in the watcher");
        current_time
    }
    #[allow(dead_code)] // TODO: Decide whether to keep this
    pub fn set_time(&mut self, new_time: PlainDateTime) {
        ADJUST_TIME.signal(new_time);
    }

    pub async fn short_vibration(&mut self) {
        self.vibrator.pulse(Duration::from_millis(15)).await;
    }

    pub async fn vibration_pattern(&mut self, on: Duration, off: Duration, repetitions: u8) {
        self.vibrator.pattern(on, off, repetitions).await;
    }

    pub async fn acceleration(&mut self) -> (i16, i16, i16) {
        self.accelerometer.get_accel_data().await.unwrap()
    }

    // TODO: move this somewhere better
    pub async fn initialize_accelerometer(&mut self) {
        self.accelerometer.init(&mut Delay).await.unwrap();
        self.accelerometer
            .set_power_mode(PowerMode::Performance)
            .await
            .unwrap();
        self.accelerometer.set_acc_conf().await.unwrap();
    }

    pub async fn disable_accelerometer(&mut self) {
        self.accelerometer
            .set_power_mode(PowerMode::Off)
            .await
            .unwrap();
    }

    pub fn battery(&self) -> (u8, bool, u32) {
        BATTERY
            .try_get()
            .expect("There will always be a valid battery value")
    }

    pub fn brightness(&self) -> BrightnessLevel {
        self.backlight.brightness()
    }

    pub fn set_brightness(&mut self, brightness: BrightnessLevel) {
        self.backlight.set_brightness(brightness);
    }

    pub fn bluetooth_connected(&self) -> bool {
        self.bluetooth_connected
    }

    pub fn rotate_screen(&mut self) {
        self.commands.push(Command::RotateScreen).unwrap();
    }

    pub fn set_bluetooth_connected(&mut self, connected: bool) {
        self.bluetooth_connected = connected;
    }

    pub fn firmware_is_validated(&self) -> bool {
        let value_ptr = VALID_BIT_ADDRESS as *const u32;
        let value = unsafe { value_ptr.read_volatile() };
        let valid = value == VALID_BIT_VALUE;
        info!(
            "[Context] Firmware Validation check at address {:#X}: value={:#X}, valid={}",
            VALID_BIT_ADDRESS, value, valid
        );
        valid
    }

    pub async fn validate_firmware(&self) {
        if !self.firmware_is_validated() {
            let mut flash = self.flash.lock().await;
            if let Err(e) = flash
                .write(VALID_BIT_ADDRESS, &VALID_BIT_VALUE.to_le_bytes())
                .await
            {
                defmt::warn!("Error validating firmware: {:?}", e);
            }
        } else {
            info!("Firmware already validated");
        }
    }
}
