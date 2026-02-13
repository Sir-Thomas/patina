use defmt::{debug, info};
use eg_seven_segment::SevenSegmentStyle;
use embedded_icon::{NewIcon, mdi::size24px as mdi};
use embedded_text::{TextBox, alignment::HorizontalAlignment};
use pinetime_bsp::touch::TouchGesture;
use time::PrimitiveDateTime;
use embedded_layout::prelude::*;
use u8g2_fonts::U8g2TextStyle;

use crate::{app_framework::prelude::*, apps::AppId};

const LG_DIGIT_HEIGHT: u32 = 120;
const LG_DIGIT_WIDTH: u32 = 45;
const LG_DIGIT_SPACING: u32 = 10;
const LG_SEGMENT_WIDTH: u32 = 10;
const SM_DIGIT_HEIGHT: u32 = 24;
const SM_DIGIT_WIDTH: u32 = 12;
const SM_DIGIT_SPACING: u32 = 4;
const SM_SEGMENT_WIDTH: u32 = 3;
const DATE_SIZE: Size = Size::new(SM_DIGIT_WIDTH * 11 + SM_DIGIT_SPACING * 10, SM_DIGIT_HEIGHT + SM_DIGIT_SPACING);
const HOURS_MINUTES_SIZE: Size = Size::new(LG_DIGIT_WIDTH * 4 + LG_DIGIT_SPACING * 5, LG_DIGIT_HEIGHT + LG_DIGIT_SPACING);
const SEC_SIZE: Size = Size::new(SM_DIGIT_WIDTH * 2 + SM_DIGIT_SPACING, SM_DIGIT_HEIGHT + SM_DIGIT_SPACING);

pub struct ClockApp{
    battery_level: u8,
    battery_charging: bool,
    bluetooth_connected: bool,
    current_time: PrimitiveDateTime,
    lg_digit_style: SevenSegmentStyle<Rgb565>,
    sm_digit_style: SevenSegmentStyle<Rgb565>,
    sm_text_style: U8g2TextStyle<Rgb565>,
    display_area: Rectangle,
    update_header: bool,
    update_date: bool,
    update_hours_minutes: bool,
    update_seconds: bool,
}

impl WatchApp for ClockApp {
    fn new() -> Self {
        let lg_digit_style = eg_seven_segment::SevenSegmentStyleBuilder::new()
            .digit_size(Size::new(LG_DIGIT_WIDTH, LG_DIGIT_HEIGHT))
            .digit_spacing(LG_DIGIT_SPACING)
            .segment_width(LG_SEGMENT_WIDTH)
            .segment_color(Rgb565::GREEN)
            .inactive_segment_color(Rgb565::new(0, 1, 0))
            .build();
        let sm_digit_style = eg_seven_segment::SevenSegmentStyleBuilder::new()
            .digit_size(Size::new(SM_DIGIT_WIDTH, SM_DIGIT_HEIGHT))
            .digit_spacing(SM_DIGIT_SPACING)
            .segment_width(SM_SEGMENT_WIDTH)
            .segment_color(Rgb565::GREEN)
            .build();
        let sm_text_style = U8g2TextStyle::new(
            u8g2_fonts::fonts::u8g2_font_spleen16x32_mr,
            Rgb565::GREEN
        );

        ClockApp {
            battery_level: 0,
            battery_charging: false,
            bluetooth_connected: false,
            current_time: PrimitiveDateTime::MIN,
            lg_digit_style,
            sm_digit_style,
            sm_text_style,
            display_area: Rectangle::new(
                Point::new(5, 5),
                Size::new(230, 230),
            ),
            update_header: true,
            update_date: true,
            update_hours_minutes: true,
            update_seconds: true,
        }
    }
    
    async fn on_start(&mut self, ctx: &mut AppContext) {
        info!("[Clock App] Starting Clock App");
        self.bluetooth_connected = ctx.bluetooth_connected();
        (self.battery_level, self.battery_charging) = ctx.battery();
        self.update_header = true;
        self.update_date = true;
        self.update_hours_minutes = true;
        self.update_seconds = true;
        self.current_time = ctx.time();
        info!(
            "[Clock App] Current Time: {:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.current_time.year(),
            self.current_time.month() as u8,
            self.current_time.day(),
            self.current_time.hour(),
            self.current_time.minute(),
            self.current_time.second()
        );
    }
    
    async fn on_stop(&mut self, _ctx: &mut AppContext) {
    }

    async fn on_event(&mut self, event: SystemEvent, ctx: &mut AppContext) -> EventResponse {
        match event {
            SystemEvent::Tick => {
                debug!("[Clock App] Handling Tick");
                let current_time = ctx.time();
                let (battery_level, battery_charging) = ctx.battery();
                debug!(
                    "[Clock App] Current Time: {:02}:{:02}:{:02}",
                    self.current_time.hour(),
                    self.current_time.minute(),
                    self.current_time.second()
                );
                if self.battery_level != battery_level || self.battery_charging != battery_charging {
                    self.battery_level = battery_level;
                    self.battery_charging = battery_charging;
                    self.update_header = true;
                }
                if self.current_time.day() != current_time.day(){
                    self.update_date = true;
                }
                if self.current_time.hour() != current_time.hour() {
                    self.update_hours_minutes = true;
                }
                if self.current_time.minute() != current_time.minute() {
                    self.update_hours_minutes = true;
                }
                if self.current_time.second() != current_time.second() {
                    self.update_seconds = true;
                }
                if self.update_date || self.update_hours_minutes || self.update_seconds {
                    self.current_time = current_time;
                    EventResponse::Rerender
                } else {
                    EventResponse::Ignore
                }
            },
            SystemEvent::Touch(event) => {
                match event.gesture {
                    TouchGesture::SwipeDown => {
                        EventResponse::SwitchApp(AppId::Flashlight)
                    }
                    _ => EventResponse::Ignore,
                }
            },
            SystemEvent::BluetoothConnected => {
                self.bluetooth_connected = true;
                self.update_header = true;
                EventResponse::Rerender
            },
            SystemEvent::BluetoothDisconnected => {
                self.bluetooth_connected = false;
                self.update_header = true;
                EventResponse::Rerender
            },
            _ => EventResponse::Ignore,
        }
    }

    async fn render(&mut self, ctx: &mut AppContext) {
        debug!("[Clock App] Rendering Clock");
        if self.update_header {
            draw_battery_icon(self.battery_level, self.battery_charging, &self.display_area, ctx).await;
            if self.bluetooth_connected {
                let bluetooth_icon = mdi::BluetoothConnect::new(Rgb565::GREEN);
                let bluetooth_icon = Image::new(&bluetooth_icon, Point::zero())
                    .align_to(&self.display_area, horizontal::Right, vertical::Top)
                    .translate(Point::new(-24, 0));
                ctx.draw_view(&bluetooth_icon, Rgb565::BLACK).await;
            } else {
                let bluetooth_icon = mdi::Bluetooth::new(Rgb565::GREEN);
                let bluetooth_icon = Image::new(&bluetooth_icon, Point::zero())
                    .align_to(&self.display_area, horizontal::Right, vertical::Top)
                    .translate(Point::new(-24, 0));
                ctx.draw_view(&bluetooth_icon, Rgb565::BLACK).await;
            }
            self.update_header = false;
        }
        if self.update_date {
            debug!("[Clock App] Updating Date");
            let mut string = String::<3>::new();
            write!(string, "{:.3}", self.current_time.weekday()).unwrap();
            let date_text = TextBox::with_alignment(
                string.as_str(),
                Rectangle::new(Point::zero(), Size::new(50, 34)),
                self.sm_text_style.clone(),
                HorizontalAlignment::Left,
            );
            let positioned_date = date_text.align_to(&self.display_area, horizontal::Left, vertical::Top);
            ctx.draw(&positioned_date, Rgb565::BLACK).await;
            let mut string = String::<11>::new();
            write!(string, "{:4}-{:02}-{:02}", self.current_time.year(), self.current_time.month() as u8, self.current_time.day()).unwrap();
            let date_text = TextBox::with_alignment(
                string.as_str(),
                Rectangle::new(Point::zero(), DATE_SIZE),
                self.sm_digit_style,
                HorizontalAlignment::Left,
            );
            let positioned_date = date_text.align_to(&self.display_area, horizontal::Left, vertical::Bottom);
            ctx.draw(&positioned_date, Rgb565::BLACK).await;
            self.update_date = false;
        }
        if self.update_hours_minutes {
            debug!("[Clock App] Updating Hours and Minutes");
            let mut string = String::<5>::new();
            write!(string, "{:02}:{:02}", to_12_hr(self.current_time.hour()), self.current_time.minute()).unwrap();
            let hours_minutes_text = TextBox::with_alignment(
                string.as_str(),
                Rectangle::new(Point::zero(), HOURS_MINUTES_SIZE),
                self.lg_digit_style,
                HorizontalAlignment::Center,
            );
            let positioned_hours = hours_minutes_text.align_to(&self.display_area, horizontal::Center, vertical::Center);
            ctx.draw(&positioned_hours, Rgb565::BLACK).await;
            self.update_hours_minutes = false;
        }
        if self.update_seconds {
            debug!("[Clock App] Updating Seconds");
            let mut string = String::<2>::new();
            write!(string, "{:02}", self.current_time.second()).unwrap();
            let seconds_text = TextBox::new(
                string.as_str(),
                Rectangle::new(Point::zero(), SEC_SIZE),
                self.sm_digit_style,
            );
            let positioned_seconds = seconds_text.align_to(&self.display_area, horizontal::Right, vertical::Bottom);
            ctx.draw(&positioned_seconds, Rgb565::BLACK).await;
            self.update_seconds = false;
        }
    }
}

fn to_12_hr(hour: u8) -> u8 {
    match hour % 12 {
        0 => 12,
        display_hour => display_hour,
    }
}

async fn draw_battery_icon(level: u8, charging: bool, display_area: &Rectangle, ctx: &mut AppContext) {
    match (level, charging) {
        (_, true) => {
            let icon = mdi::BatteryCharging::new(Rgb565::GREEN);
            let icon = Image::new(&icon, Point::zero());
            let icon = icon.align_to(display_area, horizontal::Right, vertical::Top);
            ctx.draw(&icon, Rgb565::BLACK).await;
        },
        (60..=u8::MAX, false) => {
            let icon = mdi::BatteryHigh::new(Rgb565::GREEN);
            let icon = Image::new(&icon, Point::zero());
            let icon = icon.align_to(display_area, horizontal::Right, vertical::Top);
            ctx.draw(&icon, Rgb565::BLACK).await;
        },
        (30..60, false) => {
            let icon = mdi::BatteryMedium::new(Rgb565::GREEN);
            let icon = Image::new(&icon, Point::zero());
            let icon = icon.align_to(display_area, horizontal::Right, vertical::Top);
            ctx.draw(&icon, Rgb565::BLACK).await;
        },
        (10..30, false) => {
            let icon = mdi::BatteryLow::new(Rgb565::GREEN);
            let icon = Image::new(&icon, Point::zero());
            let icon = icon.align_to(display_area, horizontal::Right, vertical::Top);
            ctx.draw(&icon, Rgb565::BLACK).await;
        },
        (0..10, false) => {
            let icon = mdi::BatteryAlert::new(Rgb565::GREEN);
            let icon = Image::new(&icon, Point::zero());
            let icon = icon.align_to(display_area, horizontal::Right, vertical::Top);
            ctx.draw(&icon, Rgb565::BLACK).await;
        },
    }
}