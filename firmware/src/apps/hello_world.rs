use crate::app_framework::prelude::*;
use buoyant::view::prelude::*;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use heapless::{String, format};
use pinetime_bsp::touch::TouchGesture;

pub struct HelloWorldApp {
    // Define any state that you want to hold here
    // State will be initialized when the app is created
    // All state will be cleaned up when on_stop is called
    // Persistent state is a potential future feature
    count: usize,
}

impl WatchApp for HelloWorldApp {
    fn new() -> Self {
        HelloWorldApp { count: 0 }
    }

    async fn on_start(&mut self, _ctx: &mut AppContext) {
        // Initialize app state
    }

    async fn on_stop(&mut self, _ctx: &mut AppContext) {
        // Clean up app state
    }

    async fn on_event(&mut self, event: SystemEvent, _ctx: &mut AppContext) -> EventResponse {
        // Look at the event and respond accordingly
        match event {
            // We will increment the counter when the screen is tapped.
            SystemEvent::Touch(touch_event) => {
                if touch_event.gesture == TouchGesture::Tap {
                    // Increment the counter
                    self.count += 1;
                    // Request a re-render to show the updated counter
                    EventResponse::Rerender
                } else {
                    // Tell the OS to do nothing for other touch gestures
                    EventResponse::Ignore
                }
            }
            // Tell the OS to do nothing for other events
            _ => EventResponse::Ignore,
        }
    }

    async fn render(&mut self, ctx: &mut AppContext) {
        // This is just something that Buoyant needs
        let mut captures = ();
        // We can put our count variable into a string to display
        let count_string: String<10> = format!("Count: {}", self.count).unwrap();
        // Build out our screen with buoyant components
        // A VStack allows us to put multiple elements on top of each other
        let screen = VStack::new((
            // We pass in our first line
            Text::new("Hello, World!", &FONT_10X20),
            // For our second line, we will use the string we made earlier
            Text::new(count_string, &FONT_10X20),
        ))
        // Align the text to the left of the display
        .with_alignment(HorizontalAlignment::Leading)
        // Make the text white
        .foreground_color(Rgb565::WHITE)
        // Put 10 pixels of padding around the edges
        .padding(Edges::All, 10);
        // Now we render the buoyant view into something embedded_graphics can display
        // We need to tell it the size of the display, give it a background color, and pass in the
        // variable we made earlier
        let view = screen.as_drawable(Size::new(240, 240), Rgb565::BLACK, &mut captures);
        // Finally, we can tell the context to draw this to the display
        ctx.draw_screen(&view).await;
    }
}
