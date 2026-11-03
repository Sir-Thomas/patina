//! Display Trait

use embedded_graphics::{pixelcolor::Rgb565, prelude::*};

use crate::ScreenOrientation;

#[allow(async_fn_in_trait)]
/// Display Trait
pub trait Display {
    /// Fill the display with the given color
    async fn fill(&mut self, color: Rgb565);

    /// Draw a drawable object to the display
    async fn draw<D: Drawable<Color = Rgb565> + Dimensions>(
        &mut self,
        drawable: &D,
        background: Rgb565,
    );

    /// Draw a drawable to the whole screen
    async fn draw_screen(&mut self, drawable: &impl Drawable<Color = Rgb565>);

    /// Put the display to sleep
    async fn sleep(&mut self);

    /// Wake the display from sleep
    async fn wake(&mut self);

    /// Get screen orientation
    fn orientation(&self) -> ScreenOrientation;

    /// Set screen orientation
    async fn set_orientation(&mut self, orientation: ScreenOrientation);
}
