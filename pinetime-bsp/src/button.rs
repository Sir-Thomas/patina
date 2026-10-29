//! Button input handler
use embassy_nrf::{Peri, gpio::{Input, Output}, peripherals::{P0_13, P0_15}};
use embassy_time::{Duration, Instant, Timer};

/// Button
pub struct Button {
    enable: Output<'static>,
    input: Input<'static>,
    polling_rate: Duration,
}

impl Button {
    /// Create a new button instance
    pub fn new(
        input_pin: Peri<'static, P0_15>,
        output_pin: Peri<'static, P0_13>,
        polling_rate: Duration,
    ) -> Self {
        let enable = Output::new(output_pin, embassy_nrf::gpio::Level::Low, embassy_nrf::gpio::OutputDrive::Standard);
        let input = Input::new(input_pin, embassy_nrf::gpio::Pull::Down);
        Button {
            enable,
            input,
            polling_rate,
        }
    }

    /// Returns true if the button is pressed
    fn is_pressed(&mut self) -> bool {
        // We must enable the pin 4 times to get a stable reading
        // The black box prevents the compiler from optimizing this away
        for _ in 0..4 {
            self.enable.set_high();
            core::hint::black_box(&self.enable);
        }
        let result = self.input.is_high();
        self.enable.set_low();
        result
    }

    /// Waits until the button is pressed
    pub async fn wait_for_press(&mut self) {
        loop {
            Timer::after(self.polling_rate).await;
            if self.is_pressed() {
                break;
            }
        }
    }

    /// Waits until the button is released
    pub async fn wait_for_release(&mut self) {
        loop {
            Timer::after(self.polling_rate).await;
            if self.is_pressed() == false {
                break;
            }
        }
    }

    /// Returns duration of the next button press
    pub async fn wait_for_press_duration(&mut self) -> Duration {
        self.wait_for_press().await;
        let start = Instant::now();
        self.wait_for_release().await;
        Instant::now() - start
    }
}