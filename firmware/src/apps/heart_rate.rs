use crate::app_framework::prelude::*;

pub struct HeartRateApp {
    heart_rate: u32,
}

impl WatchApp for HeartRateApp {
    fn new() -> Self {
        HeartRateApp { heart_rate: 0 }
    }

    async fn on_start(&mut self, ctx: &mut AppContext) {
        self.heart_rate = ctx.simple_heart_rate();
    }

    async fn on_stop(&mut self, _ctx: &mut AppContext) {
        // Clean up app state
    }

    async fn on_event(&mut self, _event: SystemEvent, _ctx: &mut AppContext) -> EventResponse {
        // Handle events
        EventResponse::Ignore
    }

    async fn render(&mut self, ctx: &mut AppContext) {
        let rate: String<4> = format!("{}", self.heart_rate).unwrap();
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
