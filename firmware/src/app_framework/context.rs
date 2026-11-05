use embassy_time::Duration;
use patina_hal::backlight::BrightnessLevel;
use time::PlainDateTime;

use crate::app_framework::prelude::*;
use crate::signals::BATTERY;

#[derive(Clone, Debug)]
pub enum Command {
    EnableAccelerometer(bool),
    RotateScreen,
    SetBrightness(BrightnessLevel),
    ValidateFirmware,
    VibrationPattern(Duration, Duration, u8),
    VibrationShort,
}

pub struct AppContext {
    pub accelerometer: Option<(i16, i16, i16)>,
    pub bluetooth_connected: bool,
    pub brightness: BrightnessLevel,
    pub firmware_is_validated: bool,
    pub screen_on: bool,
    pub time: PlainDateTime,
    pub commands: Vec<Command, 8>,
}

impl AppContext {
    pub fn new(
        brightness: BrightnessLevel,
        firmware_is_validated: bool,
        time: PlainDateTime,
    ) -> Self {
        Self {
            accelerometer: None,
            bluetooth_connected: false,
            brightness,
            firmware_is_validated,
            screen_on: true,
            time,
            commands: Vec::new(),
        }
    }

    pub fn display_is_off(&self) -> bool {
        !self.screen_on
    }

    pub fn acceleration(&mut self) -> (i16, i16, i16) {
        if let Some(data) = self.accelerometer {
            data
        } else {
            (0, 0, 0)
        }
    }

    pub fn accelerometer_start(&mut self) {
        self.commands
            .push(Command::EnableAccelerometer(true))
            .unwrap();
    }

    pub fn accelerometer_stop(&mut self) {
        self.commands
            .push(Command::EnableAccelerometer(false))
            .unwrap();
    }

    pub fn battery(&self) -> (u8, bool, u32) {
        BATTERY
            .try_get()
            .expect("There will always be a valid battery value")
    }

    pub fn set_brightness(&mut self, brightness: BrightnessLevel) {
        self.commands
            .push(Command::SetBrightness(brightness))
            .unwrap();
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

    pub fn validate_firmware(&mut self) {
        self.commands.push(Command::ValidateFirmware).unwrap();
    }

    pub fn vibration_pattern(&mut self, on: Duration, off: Duration, repetitions: u8) {
        self.commands
            .push(Command::VibrationPattern(on, off, repetitions))
            .unwrap();
    }

    pub fn vibration_short(&mut self) {
        self.commands.push(Command::VibrationShort).unwrap();
    }
}
