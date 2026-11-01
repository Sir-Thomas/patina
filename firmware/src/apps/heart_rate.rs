use crate::app_framework::prelude::*;

pub struct HeartRateApp {
    heart_rate: u32,
}

impl WatchApp for HeartRateApp {
    fn new() -> Self {
        HeartRateApp { heart_rate: 0 }
    }

    async fn on_start(&mut self, ctx: &mut AppContext) {
        ctx.start_heart_rate().await;
    }

    async fn on_stop(&mut self, ctx: &mut AppContext) {
        ctx.stop_heart_rate().await;
    }

    async fn on_event(&mut self, event: SystemEvent, ctx: &mut AppContext) -> EventResponse {
        match event {
            SystemEvent::Tick => {
                // This isn't anywhere close to accurate
                // Turns out the sensor gives raw light readings and we have to do all the
                // calculations manually...
                let hrs = ctx.read_hrs().await;
                let als = ctx.read_als().await;
                self.heart_rate = hrs - als;
                EventResponse::Rerender
            }
            _ => EventResponse::Ignore,
        }
    }

    async fn render(&mut self, ctx: &mut AppContext) {
        let rate: String<10> = format!("{}", self.heart_rate).unwrap();
        let mut captures = ();
        let screen = VStack::new((
            Text::new("Heart Rate App", &FONT_10X20),
            Text::new(rate, &FONT_10X20),
        ))
        .with_alignment(HorizontalAlignment::Leading)
        .foreground_color(Rgb565::WHITE)
        .padding(Edges::All, 10);
        let view = screen.as_drawable(Size::new(240, 240), Rgb565::BLACK, &mut captures);
        ctx.draw_screen(&view).await;
    }
}
