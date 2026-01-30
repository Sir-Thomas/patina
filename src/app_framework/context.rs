use defmt::info;
use embassy_sync::watch::DynAnonReceiver;
use embassy_time::Duration;
use pinetime_bsp::vibrator::Vibrator;
use pinetime_bsp::{backlight::BacklightController, display::DisplayController};
use time::OffsetDateTime;

use crate::signals::{ADJUST_TIME, CURRENT_TIME, TIMEOUT_DISPLAY};
use crate::app_framework::prelude::*;

pub struct AppContext {
    backlight: BacklightController,
    current_time_watcher: DynAnonReceiver<'static, OffsetDateTime>,
    display: DisplayController,
    screen_on: bool,
    vibrator: Vibrator,
}

impl AppContext {
    pub fn new(
        backlight: BacklightController,
        display: DisplayController,
        vibrator: Vibrator,
    ) -> Self {
        let current_time_watcher = CURRENT_TIME.dyn_anon_receiver();
        Self {
            backlight,
            current_time_watcher,
            display,
            screen_on: true,
            vibrator,
        }
    }

    pub fn display_is_off(&self) -> bool {
        !self.screen_on
    }

    pub fn turn_on_display(&mut self) {
        info!("[Context] Turning on display");
        TIMEOUT_DISPLAY.signal(true);
        self.screen_on = true;
        self.backlight.enable();
        // self.display.turn_on(); TODO: implement display power control
    }

    pub fn turn_off_display(&mut self) {
        info!("[Context] Turning off display");
        TIMEOUT_DISPLAY.signal(false);
        self.screen_on = false;
        self.backlight.disable();
        // self.display.turn_off(); TODO: implement display power control
    }

    pub async fn clear_display(&mut self) {
        info!("[Context] Clearing display");
        self.display.clear(Rgb565::BLACK).await;
    }

    pub async fn draw<T: Drawable<Color = Rgb565>>(
        &mut self,
        drawable: &T,
        bounds: Rectangle,
        background: Rgb565
    ) {
        // info!("[Context] Drawing Item to display");
        self.display.draw(drawable, bounds, background).await;
    }

    pub fn time(&mut self) -> OffsetDateTime {
        let current_time = self.current_time_watcher.try_get().unwrap();
        current_time
    }

    pub fn set_time(&mut self, new_time: OffsetDateTime) {
        ADJUST_TIME.signal(new_time);
    }

    pub fn adjust_time(&mut self, delta: time::Duration) {
        let new_time = self.time() + delta;
        ADJUST_TIME.signal(new_time);
    }

    pub fn reset_seconds(&mut self) {
        let mut current_time = self.time();
        current_time = current_time.truncate_to_minute();
        self.set_time(current_time);
    }

    pub async fn short_vibration(&mut self) {
        self.vibrator.pulse(Duration::from_millis(15)).await;
    }
}