use crate::app_framework::prelude::*;

pub struct StopwatchApp;

impl WatchApp for StopwatchApp {
    fn new() -> Self {
        StopwatchApp
    }

    async fn on_start(&mut self, _ctx: &mut AppContext) {
        // Initialize app state
    }

    async fn on_stop(&mut self, _ctx: &mut AppContext) {
        // Clean up app state
    }

    async fn on_event(&mut self, _event: SystemEvent, _ctx: &mut AppContext) -> EventResponse {
        // Handle events
        EventResponse::Ignore
    }

    async fn render(&mut self, ctx: &mut AppContext) {
        let mut captures = ();
        let screen = VStack::new((Text::new("Stopwatch App", &FONT_10X20),))
            .with_alignment(HorizontalAlignment::Leading)
            .foreground_color(Rgb565::WHITE)
            .padding(Edges::All, 10);
        let view = screen.as_drawable(Size::new(240, 240), Rgb565::BLACK, &mut captures);
        ctx.draw_screen(&view).await;
    }
}
