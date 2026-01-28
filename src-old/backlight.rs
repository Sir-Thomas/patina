use core::ops::Not;

use defmt::info;
use embassy_nrf::{Peri, gpio::{Level, Output, OutputDrive}, peripherals};

use crate::state::BACKLIGHT_SIGNAL;


struct BacklightController<'a> {
    low: Output<'a>,
    medium: Output<'a>,
    high: Output<'a>,
    brightness_level: BrightnessLevel,
}

impl<'a> BacklightController<'a> {
    fn new(
        backlight_low_pin: Peri<'static, peripherals::P0_14>,
        backlight_med_pin: Peri<'static, peripherals::P0_22>,
        backlight_high_pin: Peri<'static, peripherals::P0_23>,
    ) -> Self {
        Self {
            low: Output::new(backlight_low_pin, Level::High, OutputDrive::Standard),
            medium: Output::new(backlight_med_pin, Level::Low, OutputDrive::Standard),
            high: Output::new(backlight_high_pin, Level::High, OutputDrive::Standard),
            brightness_level: BrightnessLevel::Medium,
        }
    }

    fn set_brightness(&mut self, level: BrightnessLevel) {
        self.low.set_high();
        self.medium.set_high();
        self.high.set_high();
        self.brightness_level = level;
        match level {
            BrightnessLevel::Low => self.low.set_low(),
            BrightnessLevel::Medium => self.medium.set_low(),
            BrightnessLevel::High => self.high.set_low(),
        }
    }

    fn backlight_off(&mut self) {
        self.low.set_high();
        self.medium.set_high();
        self.high.set_high();
    }

    fn backlight_on(&mut self) {
        self.set_brightness(self.brightness_level);
    }
}

#[derive(Clone, Copy, defmt::Format)]
pub enum BrightnessLevel {
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, defmt::Format)]
pub enum Backlight {
    Off,
    On,
}

impl Not for Backlight {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Backlight::Off => Backlight::On,
            Backlight::On => Backlight::Off,
        }
    }
}

#[embassy_executor::task]
pub async fn backlight_task(
    backlight_low_pin: Peri<'static, peripherals::P0_14>,
    backlight_med_pin: Peri<'static, peripherals::P0_22>,
    backlight_high_pin: Peri<'static, peripherals::P0_23>,
) {
    let mut backlight_controller = BacklightController::new(backlight_low_pin, backlight_med_pin, backlight_high_pin);
    loop {
        match BACKLIGHT_SIGNAL.wait().await {
            Backlight::Off => { backlight_controller.backlight_off(); info!("Backlight Off"); },
            Backlight::On => { backlight_controller.backlight_on(); info!("Backlight On"); },
        }
    }
}