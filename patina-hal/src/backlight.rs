//! Backlight Trait

/// Brightness levels for the backlight
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BrightnessLevel {
    /// Low brightness
    #[default]
    Low,
    /// Medium brightness
    Medium,
    /// High brightness
    High,
}

/// Backlight Control
pub trait Backlight {
    /// Get current brightness level
    fn brightness(&self) -> BrightnessLevel;

    /// Set brightness level
    fn set_brightness(&mut self, level: BrightnessLevel);

    /// Enable backlight
    /// Turns the backlight back off at the previous brightness level
    fn enable(&mut self);

    /// Disable backlight
    /// Turn the backlight off completely
    fn disable(&mut self);
}
