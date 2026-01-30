use defmt::info;

use crate::app_framework::prelude::*;

pub struct SampleApp {
    // Define any state that you want to hold here
    // State will be initialized when the app is created
    // All state will be cleaned up when on_stop is called
    // Persistent state is a potential future feature
}

impl WatchApp for SampleApp {
    fn new() -> Self {
        SampleApp {}
    }

    async fn on_start(&mut self, _ctx: &mut AppContext) {
        // Initialize app state
    }
    
    async fn on_stop(&mut self, _ctx: &mut AppContext) {
        // Clean up app state
    }

    async fn on_event(&mut self, event: SystemEvent, _ctx: &mut AppContext) -> EventResponse{
        match event {
            SystemEvent::ButtonPress => {
                // Do something

                // Return "Handled" to show that you handled the event
                EventResponse::Rerender
            },
            // Catch all for any events you don't care about
            // Returns "Ignored" to show that you don't care about the event
            _ => EventResponse::Ignore,
        }
    }

    async fn render(&mut self, ctx: &mut AppContext) {
        // Render the app UI
        info!("Rendering Sample App");
        let bounds = Rectangle::new(Point::zero(), Size::new(240, 240));
        let style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
        let text = TextBox::new("Hello World!", bounds, style);
        ctx.draw(&text, bounds, Rgb565::BLACK).await;
    }
}