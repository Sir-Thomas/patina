use crate::app_framework::{AppContext, EventResponse, SystemEvent, WatchApp};
use buoyant::{match_view, view::prelude::*};
use embedded_graphics::{mono_font::ascii::FONT_10X20, pixelcolor::Rgb565, prelude::*};
use embedded_icon::{
    Icon, NewIcon,
    mdi::size48px::{
        Brightness5, Brightness6, Brightness7, Flashlight, Information, ScreenRotation,
    },
};
use heapless::{String, format};
use pinetime_bsp::{BrightnessLevel, touch::TouchGesture};

struct BrightnessIcons {
    low: Icon<Rgb565, Brightness5>,
    mid: Icon<Rgb565, Brightness6>,
    high: Icon<Rgb565, Brightness7>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsScreen {
    Main,
    Flashlight,
    Firmware,
}

pub struct SettingsApp {
    brightness_icons: BrightnessIcons,
    flashlight_icon: Icon<Rgb565, Flashlight>,
    info_icon: Icon<Rgb565, Information>,
    screen_rotation_icon: Icon<Rgb565, ScreenRotation>,
    screen: SettingsScreen,
    settings: Settings,
}

impl WatchApp for SettingsApp {
    fn new() -> Self {
        SettingsApp {
            brightness_icons: BrightnessIcons {
                low: Brightness5::new(Rgb565::BLACK),
                mid: Brightness6::new(Rgb565::BLACK),
                high: Brightness7::new(Rgb565::BLACK),
            },
            flashlight_icon: Flashlight::new(Rgb565::BLACK),
            info_icon: Information::new(Rgb565::BLACK),
            screen_rotation_icon: ScreenRotation::new(Rgb565::BLACK),
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
                    self.main_screen_touch_event(touch_event.location, ctx)
                        .await
                } else if self.screen == SettingsScreen::Firmware {
                    if (touch_event.location.x >= 10 && touch_event.location.x <= 230)
                        && (touch_event.location.y >= 180 && touch_event.location.y <= 230)
                        && !ctx.firmware_is_validated()
                    {
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
        let mut captures = ();
        let screen = match_view!(self.screen, {
            SettingsScreen::Main => self.main_screen(),
            SettingsScreen::Flashlight => {
                ctx.set_brightness(BrightnessLevel::High);
                self.flashlight_screen()
            },
            SettingsScreen::Firmware => self.firmware_screen(ctx.firmware_is_validated(), ctx.battery().2, ctx.battery().0),
        });
        let view = screen.as_drawable(Size::new(240, 240), Rgb565::BLACK, &mut captures);
        ctx.draw_screen(&view).await;
    }
}

impl SettingsApp {
    async fn main_screen_touch_event(
        &mut self,
        location: Point,
        ctx: &mut AppContext,
    ) -> EventResponse {
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
        } else {
            ctx.rotate_screen().await;
        }
        EventResponse::Rerender
    }

    fn main_screen(&self) -> impl View<Rgb565, ()> {
        VStack::new((
            HStack::new((
                self.brightness_icon(self.settings.brightness),
                self.flashlight_icon(),
            ))
            .with_spacing(10),
            HStack::new((self.screen_rotation_icon(), self.info_icon())).with_spacing(10),
        ))
        .with_spacing(10)
        .padding(Edges::All, 10)
    }

    fn brightness_icon(&self, brightness_level: BrightnessLevel) -> impl View<Rgb565, ()> {
        ZStack::new((
            Rectangle.corner_radius(25).foreground_color(Rgb565::GREEN),
            match_view!(brightness_level, {
                BrightnessLevel::Low => Image::new(&self.brightness_icons.low),
                BrightnessLevel::Medium => Image::new(&self.brightness_icons.mid),
                BrightnessLevel::High => Image::new(&self.brightness_icons.high),
            }),
        ))
    }

    fn flashlight_icon(&self) -> impl View<Rgb565, ()> {
        ZStack::new((
            Rectangle.corner_radius(25).foreground_color(Rgb565::GREEN),
            Image::new(&self.flashlight_icon),
        ))
    }

    fn screen_rotation_icon(&self) -> impl View<Rgb565, ()> {
        ZStack::new((
            Rectangle.corner_radius(25).foreground_color(Rgb565::GREEN),
            Image::new(&self.screen_rotation_icon),
        ))
    }

    fn info_icon(&self) -> impl View<Rgb565, ()> {
        ZStack::new((
            Rectangle.corner_radius(25).foreground_color(Rgb565::GREEN),
            Image::new(&self.info_icon),
        ))
    }

    fn flashlight_screen(&self) -> impl View<Rgb565, ()> {
        ZStack::new((
            Rectangle.foreground_color(Rgb565::WHITE),
            Image::new(&self.flashlight_icon),
        ))
    }

    fn firmware_screen(
        &self,
        validated: bool,
        battery_level: u32,
        battery_percent: u8,
    ) -> impl View<Rgb565, ()> {
        let string: String<100> = format!(
            "Firmware: Patina\nVersion: {}\nValidated: {}\nBattery: {}V, {}%",
            env!("CARGO_PKG_VERSION"),
            validated,
            battery_level as f32 / 1000.0,
            battery_percent,
        )
        .expect("All inputs are formattable");
        VStack::new((
            Text::new(string, &FONT_10X20).foreground_color(Rgb565::WHITE),
            Spacer::default(),
            ZStack::new((
                Capsule.foreground_color(if validated {
                    Rgb565::GREEN
                } else {
                    Rgb565::RED
                }),
                Text::new(
                    if validated {
                        "Firmware Validated"
                    } else {
                        "Validate Virmware"
                    },
                    &FONT_10X20,
                ),
            ))
            .frame_sized(220, 50)
            .with_horizontal_alignment(HorizontalAlignment::Center),
        ))
        .with_alignment(HorizontalAlignment::Leading)
        .padding(Edges::All, 10)
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
