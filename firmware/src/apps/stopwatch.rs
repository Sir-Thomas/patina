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
        let bounds = Rectangle::new(Point::zero(), Size::new(240, 240));
        let style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
        let mut string = String::<32>::new();
        write!(string, "Stopwatch App").expect("Name can be formatted");
        let text = TextBox::new(string.as_str(), bounds, style);
        ctx.draw(&text, Rgb565::BLACK).await;
    }
}
