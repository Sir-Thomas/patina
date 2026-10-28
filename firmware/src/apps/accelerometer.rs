use buoyant::view::prelude::*;
use defmt::info;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use heapless::{String, format};

use crate::app_framework::prelude::*;

pub struct AccelerometerApp {
    x: i16,
    y: i16,
    z: i16,
}

impl WatchApp for AccelerometerApp {
    fn new() -> Self {
        AccelerometerApp { x: 0, y: 0, z: 0 }
    }

    async fn on_start(&mut self, ctx: &mut AppContext) {
        ctx.initialize_accelerometer().await;
    }

    async fn on_stop(&mut self, ctx: &mut AppContext) {
        ctx.disable_accelerometer().await;
    }

    async fn on_event(&mut self, event: SystemEvent, ctx: &mut AppContext) -> EventResponse {
        match event {
            SystemEvent::Tick => {
                (self.x, self.y, self.z) = ctx.acceleration().await;
                info!("x {}, y {}, z {}", self.x, self.y, self.z);
                EventResponse::Rerender
            }
            _ => EventResponse::Ignore,
        }
    }

    async fn render(&mut self, ctx: &mut AppContext) {
        let mut captures = ();
        let x_string: String<9> = format!("X: {}", self.x).unwrap();
        let y_string: String<9> = format!("Y: {}", self.y).unwrap();
        let z_string: String<9> = format!("Z: {}", self.z).unwrap();
        let screen = VStack::new((
            Text::new(x_string, &FONT_10X20),
            Text::new(y_string, &FONT_10X20),
            Text::new(z_string, &FONT_10X20),
        ))
        .with_alignment(HorizontalAlignment::Leading)
        .foreground_color(Rgb565::WHITE)
        .padding(Edges::All, 10);
        let view = screen.as_drawable(Size::new(240, 240), Rgb565::BLACK, &mut captures);
        ctx.draw_screen(&view).await;
    }
}
