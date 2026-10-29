//! Vibrator motor control

use embassy_nrf::{Peri, gpio::{Level, Output, OutputDrive}, peripherals::P0_16};
use embassy_time::{Duration, Timer};

/// Vibrator motor controller
pub struct Vibrator {
    output: Output<'static>,
}

impl Vibrator {
    /// Create a new vibrator instance
    pub fn new(output_pin: Peri<'static, P0_16>) -> Self {
        let output = Output::new(
            output_pin,
            Level::High,
            OutputDrive::Standard);
        Vibrator { output }
    }

    /// Pulse the vibrator for a specified duration
    pub async fn pulse(&mut self, duration: Duration) {
        self.output.set_low();
        Timer::after(duration).await;
        self.output.set_high();
    }

    /// Create a vibration pattern with on/off cycles
    /// 
    /// # Arguments
    /// * `on_duration` - How long to vibrate during each pulse
    /// * `off_duration` - How long to wait between pulses
    /// * `repetitions` - Number of pulses to perform
    pub async fn pattern(
        &mut self,
        on_duration: Duration,
        off_duration: Duration,
        repetitions: u8,
    ) {
        for _ in 0..repetitions {
            self.pulse(on_duration).await;
            Timer::after(off_duration).await;
        }
    }
}