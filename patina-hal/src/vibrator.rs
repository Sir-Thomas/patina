//! Vibrator Trait

use embassy_time::Duration;

#[allow(async_fn_in_trait)]
/// Vibrator Trait
pub trait Vibrator {
    /// Pulse the vibrator for a specified duration
    async fn pulse(&mut self, duration: Duration);

    /// Create a vibration pattern with on/off cycles
    ///
    /// # Arguments
    /// * `on_duration` - How long to vibrate during each pulse
    /// * `off_duration` - How long to wait between pulses
    /// * `repetitions` - Number of pulses to perform
    async fn pattern(&mut self, on_duration: Duration, off_duration: Duration, repetitions: u8);
}
