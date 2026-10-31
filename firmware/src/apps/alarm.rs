use embedded_icon::{
    mdi::size48px::{MenuDown, MenuUp},
    prelude::*,
};
use time::{SignedDuration, Time};
use u8g2_fonts::{FontRenderer, fonts};

use crate::{app_framework::prelude::*, signals::SET_ALARM};

static LARGE_FONT: FontRenderer = FontRenderer::new::<fonts::u8g2_font_logisoso78_tn>();

pub struct AlarmApp {
    enabled: bool,
    time: Time,
    up_arrow: Icon<Rgb565, MenuUp>,
    down_arrow: Icon<Rgb565, MenuDown>,
}

impl WatchApp for AlarmApp {
    fn new() -> Self {
        AlarmApp {
            enabled: false,
            time: Time::from_hms(6, 0, 0).unwrap(),
            up_arrow: MenuUp::new(Rgb565::GREEN),
            down_arrow: MenuDown::new(Rgb565::GREEN),
        }
    }

    async fn on_start(&mut self, _ctx: &mut AppContext) {
        let signal = SET_ALARM.anon_receiver().try_get();
        if let Some((time, enabled)) = signal {
            self.enabled = enabled;
            self.time = time;
        }
    }

    async fn on_stop(&mut self, _ctx: &mut AppContext) {
        // Clean up app state
    }

    async fn on_event(&mut self, event: SystemEvent, _ctx: &mut AppContext) -> EventResponse {
        match event {
            SystemEvent::Touch(touch_event)
                if touch_event.location.x > 180 && touch_event.location.y > 180 =>
            {
                self.enabled = !self.enabled;
            }
            SystemEvent::Touch(touch_event)
                if touch_event.location.x < 120 && touch_event.location.y < 100 =>
            {
                self.time += SignedDuration::hours(1);
            }
            SystemEvent::Touch(touch_event)
                if touch_event.location.x > 120 && touch_event.location.y < 100 =>
            {
                self.time += SignedDuration::minutes(1);
            }
            SystemEvent::Touch(touch_event)
                if touch_event.location.x < 120 && touch_event.location.y > 100 =>
            {
                self.time -= SignedDuration::hours(1);
            }
            SystemEvent::Touch(touch_event)
                if touch_event.location.x > 120 && touch_event.location.y > 100 =>
            {
                self.time -= SignedDuration::minutes(1);
            }
            _ => return EventResponse::Ignore,
        }
        self.set_alarm();
        EventResponse::Rerender
    }

    async fn render(&mut self, ctx: &mut AppContext) {
        let mut captures = ();
        let screen = VStack::new((
            Spacer::default(),
            self.up_arrows(),
            hours_minutes(self.time),
            self.down_arrows(),
            Spacer::default(),
            enable(self.enabled),
        ))
        .with_alignment(HorizontalAlignment::Center);
        let view = screen.as_drawable(Size::new(240, 240), Rgb565::BLACK, &mut captures);
        ctx.draw_screen(&view).await;
    }
}

impl AlarmApp {
    fn up_arrows(&self) -> impl View<Rgb565, ()> {
        HStack::new((
            Image::new(&self.up_arrow),
            Spacer::default(),
            Image::new(&self.up_arrow),
        ))
        .foreground_color(Rgb565::GREEN)
        .padding(Edges::Horizontal, 40)
    }

    fn down_arrows(&self) -> impl View<Rgb565, ()> {
        HStack::new((
            Image::new(&self.down_arrow),
            Spacer::default(),
            Image::new(&self.down_arrow),
        ))
        .foreground_color(Rgb565::GREEN)
        .padding(Edges::Horizontal, 40)
    }

    fn set_alarm(&self) {
        let sender = SET_ALARM.sender();
        sender.send((self.time, self.enabled));
    }
}

fn hours_minutes(time: Time) -> impl View<Rgb565, ()> {
    let hours = time.hour();
    let minutes = time.minute();
    let text: String<5> = format!("{:02}:{:02}", hours, minutes).unwrap();
    Text::new(text, &LARGE_FONT)
        .foreground_color(Rgb565::GREEN)
        .background_color(Rgb565::BLACK, Rectangle)
}

fn enable(status: bool) -> impl View<Rgb565, ()> {
    let alignment = if status {
        Alignment::Trailing
    } else {
        Alignment::Leading
    };
    let color = if status { Rgb565::GREEN } else { Rgb565::RED };
    HStack::new((
        Text::new("[Alarm] Enabled", &FONT_10X20).foreground_color(Rgb565::WHITE),
        Spacer::default(),
        Capsule
            .foreground_color(color)
            .overlay(
                alignment,
                Circle
                    .foreground_color(Rgb565::WHITE)
                    .padding(Edges::All, 2),
            )
            .frame_sized(50, 25),
    ))
}
