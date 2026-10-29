//! Backlight controller
use embassy_nrf::{Peri, gpio::{Level, Output, OutputDrive}, peripherals};

/// Backlight controller
pub struct BacklightController {
    low: Output<'static>,
    medium: Output<'static>,
    high: Output<'static>,
    brightness_level: BrightnessLevel,
}

impl BacklightController {
    /// Create a new backlight controller instance
    pub fn new(
        backlight_low_pin: Peri<'static, peripherals::P0_14>,
        backlight_med_pin: Peri<'static, peripherals::P0_22>,
        backlight_high_pin: Peri<'static, peripherals::P0_23>,
    ) -> Self {
        Self {
            low: Output::new(backlight_low_pin, Level::High, OutputDrive::Standard),
            medium: Output::new(backlight_med_pin, Level::High, OutputDrive::Standard),
            high: Output::new(backlight_high_pin, Level::High, OutputDrive::Standard),
            brightness_level: BrightnessLevel::default(),
        }
    }

    /// Get the current brightness level
    pub fn brightness(&self) -> BrightnessLevel {
        self.brightness_level
    }

    /// Set the brightness level
    pub fn set_brightness(&mut self, level: BrightnessLevel) {
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

    /// Enable the backlight at the current brightness level
    pub fn enable(&mut self) {
        self.set_brightness(self.brightness_level);
    }

    /// Disable the backlight
    pub fn disable(&mut self) {
        self.low.set_high();
        self.medium.set_high();
        self.high.set_high();
    }
}

/// Brightness levels for the backlight
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum BrightnessLevel {
    /// Low brightness
    #[default]
    Low,
    /// Medium brightness
    Medium,
    /// High brightness
    High,
}