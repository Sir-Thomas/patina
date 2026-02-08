use defmt::info;
use embassy_sync::watch::DynAnonReceiver;
use embassy_time::Duration;
use embedded_layout::View;
use pinetime_bsp::BrightnessLevel;
use pinetime_bsp::vibrator::Vibrator;
use pinetime_bsp::{backlight::BacklightController, display::DisplayController};
use time::PrimitiveDateTime;

use crate::signals::{ADJUST_TIME, CURRENT_TIME, CHANGE_DISPLAY_STATE};
use crate::app_framework::prelude::*;

pub struct AppContext {
    backlight: BacklightController,
    bluetooth_connected: bool,
    current_time_watcher: DynAnonReceiver<'static, PrimitiveDateTime>,
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
            bluetooth_connected: false,
            current_time_watcher,
            display,
            screen_on: true,
            vibrator,
        }
    }

    pub fn display_is_off(&self) -> bool {
        !self.screen_on
    }

    pub async fn turn_on_display(&mut self) {
        info!("[Context] Turning on display");
        CHANGE_DISPLAY_STATE.signal(true);
        self.screen_on = true;
        self.backlight.enable();
        self.display.wake().await;
    }

    pub async fn turn_off_display(&mut self) {
        info!("[Context] Turning off display");
        CHANGE_DISPLAY_STATE.signal(false);
        self.screen_on = false;
        self.backlight.disable();
        self.clear_display().await;
        self.display.sleep().await;
    }

    pub async fn clear_display(&mut self) {
        info!("[Context] Clearing display");
        self.display.clear(Rgb565::BLACK).await;
    }

    pub async fn draw<T: Drawable<Color = Rgb565> + Dimensions>(
        &mut self,
        drawable: &T,
        background: Rgb565
    ) {
        // info!("[Context] Drawing Item to display");
        self.display.draw(drawable, background).await;
    }

    pub async fn draw_view<T: Drawable<Color = Rgb565> + View>(
        &mut self,
        view: &T,
        background: Rgb565
    ) {
        // info!("[Context] Drawing View to display");
        self.display.draw_view(view, background).await;
    }

    pub fn time(&mut self) -> PrimitiveDateTime {
        let current_time = self.current_time_watcher.try_get().unwrap();
        current_time
    }

    pub fn set_time(&mut self, new_time: PrimitiveDateTime) {
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
    
    pub fn brightness(&self) -> BrightnessLevel {
        self.backlight.brightness()
    }

    pub fn set_brightness(&mut self, brightness: BrightnessLevel) {
        self.backlight.set_brightness(brightness);
    }

    pub fn bluetooth_connected(&self) -> bool {
        self.bluetooth_connected
    }

    pub fn set_bluetooth_connected(&mut self, connected: bool) {
        self.bluetooth_connected = connected;
    }
}