use embedded_icon::{NewIcon, mdi};
use embedded_text::{alignment::{HorizontalAlignment, VerticalAlignment}, style::TextBoxStyleBuilder};
use pinetime_bsp::{BrightnessLevel, touch::TouchGesture};

use crate::app_framework::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsScreen {
    Main,
    Flashlight,
    Firmware,
}

pub struct SettingsApp {
    screen: SettingsScreen,
    settings: Settings,
}

impl WatchApp for SettingsApp {
    fn new() -> Self {
        SettingsApp {
            screen: SettingsScreen::Main,
            settings: Settings::default(),
        }
    }
    
    async fn on_start(&mut self, ctx: &mut AppContext) {
        self.settings.load(ctx);
    }
    
    async fn on_stop(&mut self, ctx: &mut AppContext) {
        ctx.set_brightness(self.settings.brightness);
    }

    async fn on_event(&mut self, event: SystemEvent, ctx: &mut AppContext) -> EventResponse {
        match event {
            SystemEvent::Touch(touch_event) => {
                if touch_event.gesture == TouchGesture::SwipeUp {
                    EventResponse::CloseApp
                } else if self.screen == SettingsScreen::Main {
                    self.main_screen_touch_event(touch_event.location, ctx).await
                } else if self.screen == SettingsScreen::Firmware {
                    if (touch_event.location.x >= 10 && touch_event.location.x <= 230)
                    && (touch_event.location.y >= 180 && touch_event.location.y <= 230) 
                    && !ctx.firmware_is_validated() {
                        ctx.validate_firmware().await;
                        EventResponse::Rerender
                    } else {
                        self.screen = SettingsScreen::Main;
                        EventResponse::Rerender
                    }
                } else if self.screen == SettingsScreen::Flashlight {
                    self.screen = SettingsScreen::Main;
                    EventResponse::Rerender
                } else {
                    EventResponse::Ignore
                }
            }
            _ => EventResponse::Ignore,
        }
    }

    async fn render(&mut self, ctx: &mut AppContext) {
        ctx.set_brightness(self.settings.brightness);
        let display_area = Rectangle::new(Point::zero(), Size::new(240, 240));
        match self.screen {
            SettingsScreen::Main => {
                self.draw_main_screen(ctx, &display_area).await;
            },
            SettingsScreen::Flashlight => {
                ctx.set_brightness(BrightnessLevel::High);
                ctx.fill_display(Rgb565::WHITE).await;
                let flashlight_icon = mdi::size48px::Flashlight::new(Rgb565::CSS_LIGHT_GRAY);
                let flashlight_icon = Image::new(&flashlight_icon, Point::zero())
                    .align_to(&display_area, horizontal::Center, vertical::Center);
                ctx.draw(&flashlight_icon, Rgb565::WHITE).await;
            }
            SettingsScreen::Firmware => {
                let validated = ctx.firmware_is_validated();
                let bounds = Rectangle::new(Point::new(10, 10), Size::new(220, 220));
                let style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
                let mut string = String::<256>::new();
                write!(
                    string, "Firmware: {}\nVersion: {}\nValidated: {}\nBattery: {}V",
                    env!("CARGO_PKG_NAME"),
                    env!("CARGO_PKG_VERSION"),
                    validated,
                    ctx.battery().2 as f32 / 1000.0,
                ).unwrap();
                let text = TextBox::new(string.as_str(), bounds, style);
                ctx.draw(&text, Rgb565::BLACK).await;
                if !validated {
                    let style = MonoTextStyle::new(&FONT_10X20, Rgb565::BLACK);
                    let centered = TextBoxStyleBuilder::new()
                        .alignment(HorizontalAlignment::Center)
                        .vertical_alignment(VerticalAlignment::Middle)
                        .build();
                    let validate = TextBox::with_textbox_style(
                        "Validate Firmware",
                        Rectangle::new(Point::new(10, 180),
                        Size::new(220, 50)),
                        style,
                        centered,
                    );
                    ctx.draw(&validate, Rgb565::GREEN).await;
                }
            },
        }
    }
}

impl SettingsApp {
    async fn main_screen_touch_event(&mut self, location: Point, ctx: &mut AppContext) -> EventResponse {
        if location.x <= 120 && location.y <= 120 {
            ctx.short_vibration().await;
            self.settings.brightness = match self.settings.brightness {
                BrightnessLevel::Low => BrightnessLevel::Medium,
                BrightnessLevel::Medium => BrightnessLevel::High,
                BrightnessLevel::High => BrightnessLevel::Low,
            };
        } else if location.x > 120 && location.y <= 120 {
            self.screen = SettingsScreen::Flashlight;
        } else if location.x > 120 && location.y > 120 {
            self.screen = SettingsScreen::Firmware;
        }
        EventResponse::Rerender
    }

    async fn draw_main_screen(&self, ctx: &mut AppContext, display_area: &Rectangle) {
        const OFFSET: i32 = 60;
        ctx.clear_display().await;
        self.draw_brightness_icon(self.settings.brightness, ctx, display_area, OFFSET).await;
        let flashlight_icon = mdi::size48px::Flashlight::new(Rgb565::CSS_LIGHT_GRAY);
        let flashlight_icon = Image::new(&flashlight_icon, Point::zero())
            .align_to(display_area, horizontal::Center, vertical::Center)
            .translate(Point::new(OFFSET, -OFFSET));
        ctx.draw(&flashlight_icon, Rgb565::BLACK).await;
        let info_icon = mdi::size48px::Information::new(Rgb565::CSS_LIGHT_GRAY);
        let info_icon = Image::new(&info_icon, Point::zero())
            .align_to(display_area, horizontal::Center, vertical::Center)
            .translate(Point::new(OFFSET, OFFSET));
        ctx.draw(&info_icon, Rgb565::BLACK).await;
    }

    async fn draw_brightness_icon(
        &self,
        brightness_level: BrightnessLevel,
        ctx: &mut AppContext,
        display_area: &Rectangle,
        offset: i32
    ) {
        match brightness_level {
            BrightnessLevel::Low =>{
                let brightness_icon = mdi::size48px::Brightness5::new(Rgb565::CSS_LIGHT_GRAY);
                let brightness_icon = Image::new(&brightness_icon, Point::zero())
                    .align_to(display_area, horizontal::Center, vertical::Center)
                    .translate(Point::new(-offset, -offset));
                ctx.draw(&brightness_icon, Rgb565::BLACK).await;
            }
            BrightnessLevel::Medium =>{
                let brightness_icon = mdi::size48px::Brightness6::new(Rgb565::CSS_LIGHT_GRAY);
                let brightness_icon = Image::new(&brightness_icon, Point::zero())
                    .align_to(display_area, horizontal::Center, vertical::Center)
                    .translate(Point::new(-offset, -offset));
                ctx.draw(&brightness_icon, Rgb565::BLACK).await;
            }
            BrightnessLevel::High =>{
                let brightness_icon = mdi::size48px::Brightness7::new(Rgb565::CSS_LIGHT_GRAY);
                let brightness_icon = Image::new(&brightness_icon, Point::zero())
                    .align_to(display_area, horizontal::Center, vertical::Center)
                    .translate(Point::new(-offset, -offset));
                ctx.draw(&brightness_icon, Rgb565::BLACK).await;
            }
        }
    }
}

#[derive(Default)]
struct Settings {
    brightness: BrightnessLevel,
}

impl Settings {
    fn load(&mut self, ctx: &mut AppContext) {
        self.brightness = ctx.brightness();
    }
}