//! Button Trait

use embassy_time::Duration;

#[allow(async_fn_in_trait)]
/// Button
pub trait Button {
    /// Returns true if the button is pressed
    fn is_pressed(&mut self) -> bool;

    /// Waits until the button is pressed
    async fn wait_for_press(&mut self);

    /// Waits until the button is released
    async fn wait_for_release(&mut self);

    /// Returns duration of the next button press
    async fn wait_for_press_duration(&mut self) -> Duration;
}
