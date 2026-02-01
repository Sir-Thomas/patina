use embedded_graphics::primitives::PrimitiveStyle;
use pinetime_bsp::{BrightnessLevel, touch::TouchGesture};

use crate::app_framework::prelude::*;

pub struct FlashlightApp {
    is_on: bool,
    previous_brightness: BrightnessLevel,
}

impl WatchApp for FlashlightApp {
    fn new() -> Self {
        FlashlightApp { 
            is_on: true,
            previous_brightness: BrightnessLevel::Medium,
        }
    }

    async fn on_start(&mut self, ctx: &mut AppContext) {
        self.previous_brightness = ctx.brightness();
        self.is_on = true;
    }

    async fn on_stop(&mut self, ctx: &mut AppContext) {
        ctx.set_brightness(self.previous_brightness);
        self.is_on = false;
    }

    async fn on_event(
        &mut self,
        event: crate::app_framework::SystemEvent,
        _ctx: &mut AppContext
    ) -> crate::app_framework::EventResponse {
        match event {
            SystemEvent::Touch(touch_event) => {
                if touch_event.gesture == TouchGesture::SwipeUp {
                    EventResponse::CloseApp
                } else {
                    EventResponse::Ignore
                }
            }
            _ => {
                EventResponse::Ignore
            }
        }
    }

    async fn render(&mut self, ctx: &mut AppContext) {
        if self.is_on {
            ctx.set_brightness(BrightnessLevel::High);
            ctx.draw(
                &Rectangle::new(Point::new(0, 0), Size::new(240, 240))
                    .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE)),
                Rectangle::new(Point::new(0, 0), Size::new(240, 240)),
                Rgb565::WHITE,
            )
            .await;
        } else {
            ctx.clear_display().await;
        }
    }
}