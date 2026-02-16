use pinetime_bsp::touch::TouchGesture;

use crate::app_framework::prelude::*;

pub struct HelloWorldApp {
    // Define any state that you want to hold here
    // State will be initialized when the app is created
    // All state will be cleaned up when on_stop is called
    // Persistent state is a potential future feature
    count: usize,
}

impl WatchApp for HelloWorldApp {
    fn new() -> Self {
        HelloWorldApp {
            count: 0,
        }
    }

    async fn on_start(&mut self, _ctx: &mut AppContext) {
        // Initialize app state
    }
    
    async fn on_stop(&mut self, _ctx: &mut AppContext) {
        // Clean up app state
    }

    async fn on_event(&mut self, event: SystemEvent, _ctx: &mut AppContext) -> EventResponse{
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
            },
            // Tell the OS to do nothing for other events
            _ => EventResponse::Ignore,
        }
    }

    async fn render(&mut self, ctx: &mut AppContext) {
        // create a bounding box the size of the screen
        let bounds = Rectangle::new(Point::zero(), Size::new(240, 240));
        // set up our text style
        let style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
        // write our string
        let mut string = String::<32>::new();
        write!(string, "Hello, World!\nCount: {}", self.count).unwrap();
        // create our text box
        let text = TextBox::new(string.as_str(), bounds, style);
        // draw the text to the screen
        ctx.draw(&text, Rgb565::BLACK).await;
    }
}