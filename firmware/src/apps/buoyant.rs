use core::fmt::Write as _;
use buoyant::{if_view, match_view, view::prelude::*};
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use embedded_icon::{Icon, NewIcon, mdi::size24px::{BatteryAlertVariantOutline, BatteryCharging, BatteryHigh, BatteryLow, BatteryMedium, Bluetooth, BluetoothConnect}};
use heapless::String;
use time::PrimitiveDateTime;
use u8g2_fonts::{fonts, FontRenderer};

use crate::app_framework::{AppContext, EventResponse, SystemEvent, WatchApp};
static SPLEEN: FontRenderer = FontRenderer::new::<fonts::u8g2_font_spleen16x32_mr>();
static SEVEN_SEGMENT_SM: FontRenderer = FontRenderer::new::<fonts::u8g2_font_7_Seg_33x19_mn>();
static SEVEN_SEGMENT_LG: FontRenderer = FontRenderer::new::<fonts::u8g2_font_logisoso78_tn>();

struct BluetoothIcons {
    connected: Icon<Rgb565, BluetoothConnect>,
    disconnected: Icon<Rgb565, Bluetooth>,
}

struct BatteryIcons {
    charging: Icon<Rgb565, BatteryCharging>,
    high: Icon<Rgb565, BatteryHigh>,
    medium: Icon<Rgb565, BatteryMedium>,
    low: Icon<Rgb565, BatteryLow>,
    critical: Icon<Rgb565, BatteryAlertVariantOutline>,
}

pub struct BuoyantApp {
    current_time: PrimitiveDateTime,
    bluetooth_connected: bool,
    bluetooth_icons: BluetoothIcons,
    battery_level: u8,
    battery_charging: bool,
    battery_icons: BatteryIcons,
}

impl WatchApp for BuoyantApp {
    fn new() -> Self {
        BuoyantApp {
            current_time: PrimitiveDateTime::MIN,
            bluetooth_connected: false,
            bluetooth_icons: BluetoothIcons {
                connected: BluetoothConnect::new(Rgb565::GREEN),
                disconnected: Bluetooth::new(Rgb565::GREEN),
            },
            battery_level: 100,
            battery_charging: false,
            battery_icons: BatteryIcons {
                charging: BatteryCharging::new(Rgb565::GREEN),
                high: BatteryHigh::new(Rgb565::GREEN),
                medium: BatteryMedium::new(Rgb565::GREEN),
                low: BatteryLow::new(Rgb565::GREEN),
                critical: BatteryAlertVariantOutline::new(Rgb565::RED),
            },
        }
    }

    async fn on_start(&mut self, ctx: &mut AppContext) {
        self.current_time = ctx.time();
        (self.battery_level, self.battery_charging, _) = ctx.battery();
        self.bluetooth_connected = ctx.bluetooth_connected();
    }
    
    async fn on_stop(&mut self, _ctx: &mut AppContext) {
        // Clean up app state
    }

    async fn on_event(&mut self, event: SystemEvent, ctx: &mut AppContext) -> EventResponse{
        match event {
            SystemEvent::Tick => {
                let current_time = ctx.time();
                let (battery_level, battery_charging, _) = ctx.battery();
                if current_time.second() != self.current_time.second() ||
                   battery_level != self.battery_level ||
                   battery_charging != self.battery_charging {
                    self.current_time = current_time;
                    self.battery_level = battery_level;
                    self.battery_charging = battery_charging;
                    EventResponse::Rerender
                } else {
                    EventResponse::Ignore
                }
            },
            SystemEvent::BluetoothConnected => {
                self.bluetooth_connected = true;
                EventResponse::Rerender
            },
            SystemEvent::BluetoothDisconnected => {
                self.bluetooth_connected = false;
                EventResponse::Rerender
            },
            _ => EventResponse::Ignore,
        }
    }

    async fn render(&mut self, ctx: &mut AppContext) {
        let mut captures = ();
        let clock = self.clock();
        let view = clock.as_drawable(Size::new(240,240), Rgb565::WHITE, &mut captures);
        ctx.draw_screen(&view).await;
    }
}

impl BuoyantApp {
    fn clock(&self) -> impl View<Rgb565, ()> {
        VStack::new((
            self.header().frame(),
            self.hours_minutes()
                .flex_infinite_height(VerticalAlignment::Center),
            self.footer(),
        ))
    }

    fn header(&self) -> impl View<Rgb565, ()> {
        let mut weekday: String::<3> = String::new();
        write!(weekday, "{:.3}", self.current_time.weekday()).unwrap();
        HStack::new((
            Text::new(weekday, &SPLEEN).foreground_color(Rgb565::GREEN)
                .background_color(Rgb565::BLACK, Rectangle),
            Spacer::default(),
            self.bluetooth_icon().background_color(Rgb565::BLACK, Rectangle),
            self.battery_icon().background_color(Rgb565::BLACK, Rectangle),
        ))
    }

    fn bluetooth_icon(&self) -> impl View<Rgb565, ()> {
        if_view!((self.bluetooth_connected) {
            Image::new(&self.bluetooth_icons.connected)
        } else {
            Image::new(&self.bluetooth_icons.disconnected)
        })
    }

    fn battery_icon(&self) -> impl View<Rgb565, ()> {
        if_view!((self.battery_charging) {
            Image::new(&self.battery_icons.charging)
        } else {
            self.battery_level_icon()
        })
    }

    fn battery_level_icon(&self) -> impl View<Rgb565, ()> {
        match_view!(self.battery_level, {
            0..=20 => Image::new(&self.battery_icons.critical),
            21..=40 => Image::new(&self.battery_icons.low),
            41..=70 => Image::new(&self.battery_icons.medium),
            _ => Image::new(&self.battery_icons.high),
        })
    }

    fn hours_minutes(&self) -> impl View<Rgb565, ()> {
        let hours = to_12_hr(self.current_time.hour());
        let minutes = self.current_time.minute();
        let mut text: String<5> = String::new();
        write!(text, "{:02}:{:02}", hours, minutes).unwrap();
        Text::new(text, &SEVEN_SEGMENT_LG)
            .foreground_color(Rgb565::GREEN)
            .background_color(Rgb565::BLACK, Rectangle)
    }

    fn footer(&self) -> impl View<Rgb565, ()> {
        let mut date: String<10> = String::new();
        write!(
            date,
            "{:4}-{:02}-{:02}",
            self.current_time.year().abs(),
            self.current_time.month() as u8,
            self.current_time.day()
        ).unwrap();
        let mut seconds = String::<2>::new();
        write!(seconds, "{:02}", self.current_time.second()).unwrap();
        HStack::new((
            Text::new(date, &SEVEN_SEGMENT_SM)
                .foreground_color(Rgb565::GREEN)
                .background_color(Rgb565::BLACK, Rectangle),
            Spacer::default(),
            Text::new(seconds, &SEVEN_SEGMENT_SM)
                .foreground_color(Rgb565::GREEN)
                .background_color(Rgb565::BLACK, Rectangle),
        ))
    }
}


fn to_12_hr(hour: u8) -> u8 {
    match hour % 12 {
        0 => 12,
        display_hour => display_hour,
    }
}