use buoyant::if_view;
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    watch::{Sender, Watch},
};
use embassy_time::{Duration, Instant};
use u8g2_fonts::{FontRenderer, fonts};

use crate::app_framework::prelude::*;

const WATCHERS: usize = 0;
static STATE: Watch<ThreadModeRawMutex, State, WATCHERS> = Watch::new();

static SEVEN_SEGMENT: FontRenderer = FontRenderer::new::<fonts::u8g2_font_7_Seg_41x21_mn>();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum State {
    Stopped(Duration),
    Running(Instant),
}

pub struct StopwatchApp {
    state: State,
    sender: Sender<'static, ThreadModeRawMutex, State, WATCHERS>,
    last_render: Instant,
}

impl WatchApp for StopwatchApp {
    fn new() -> Self {
        let sender = STATE.sender();
        let mut receiver = STATE.anon_receiver();
        let state = receiver.try_get().unwrap_or_else(|| {
            sender.send(State::Stopped(Duration::from_secs(0)));
            State::Stopped(Duration::from_secs(0))
        });
        StopwatchApp {
            state,
            sender,
            last_render: Instant::MIN,
        }
    }

    async fn on_start(&mut self, _ctx: &mut AppContext) {}

    async fn on_stop(&mut self, _ctx: &mut AppContext) {}

    async fn on_event(&mut self, event: SystemEvent, _ctx: &mut AppContext) -> EventResponse {
        match event {
            SystemEvent::Tick => match self.state {
                State::Running(_) => {
                    if (Instant::now() - self.last_render).as_millis() > 250 {
                        self.last_render = Instant::now();
                        EventResponse::Rerender
                    } else {
                        EventResponse::Ignore
                    }
                }
                State::Stopped(_) => EventResponse::Ignore,
            },
            SystemEvent::Touch(t) if t.location.x > 120 && t.location.y > 120 => {
                self.toggle();
                EventResponse::Rerender
            }
            SystemEvent::Touch(t) if t.location.x < 120 && t.location.y > 120 => {
                self.reset();
                EventResponse::Rerender
            }
            _ => EventResponse::Ignore,
        }
    }

    async fn render(&mut self, ctx: &mut AppContext) {
        let mut captures = ();
        let screen = self.screen();
        let view = screen.as_drawable(Size::new(240, 240), Rgb565::BLACK, &mut captures);
        ctx.draw_screen(&view).await;
    }
}

impl StopwatchApp {
    fn screen(&self) -> impl View<Rgb565, ()> {
        VStack::new((
            self.time().flex_infinite_height(VerticalAlignment::Center),
            self.buttons(),
        ))
        .with_alignment(HorizontalAlignment::Center)
        .padding(Edges::All, 10)
    }

    fn time(&self) -> impl View<Rgb565, ()> {
        let time = match self.state {
            State::Stopped(t) => t,
            State::Running(start) => Instant::now() - start,
        };
        let time_string: String<20> = format!(
            "{:02}:{:02}.{:02}",
            time.as_secs() / 60,
            time.as_secs() % 60,
            (time.as_millis() % 1000) / 10,
        )
        .unwrap();
        Text::new(time_string, &SEVEN_SEGMENT).foreground_color(Rgb565::GREEN)
    }

    fn buttons(&self) -> impl View<Rgb565, ()> {
        let running = match self.state {
            State::Running(_) => true,
            State::Stopped(_) => false,
        };
        let show_reset = match self.state {
            State::Stopped(t) if t.as_millis() == 0 => false,
            State::Running(_) => false,
            _ => true,
        };
        HStack::new((
            if_view!((show_reset) {
                ZStack::new((
                    Capsule.foreground_color(Rgb565::RED),
                    Text::new("Reset", &FONT_10X20).foreground_color(Rgb565::WHITE),
                ))
            } else {
                Capsule.foreground_color(Rgb565::BLACK)
            }),
            if_view!((running) {
                ZStack::new((
                    Capsule.foreground_color(Rgb565::RED),
                    Text::new("Stop", &FONT_10X20).foreground_color(Rgb565::WHITE),
                ))
            } else {
                ZStack::new((
                    Capsule.foreground_color(Rgb565::GREEN),
                    Text::new("Start", &FONT_10X20).foreground_color(Rgb565::BLACK),
                ))
            }),
        ))
    }

    fn toggle(&mut self) {
        match self.state {
            State::Running(start) => {
                self.state = State::Stopped(Instant::now() - start);
            }
            State::Stopped(time) => {
                self.state = State::Running(Instant::now() - time);
            }
        }
        self.sender.send(self.state);
    }

    fn reset(&mut self) {
        self.state = State::Stopped(Duration::from_secs(0));
        self.sender.send(self.state);
    }
}
