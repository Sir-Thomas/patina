use crate::app_framework::prelude::*;
use embassy_time::Duration;
use embedded_icon::{mdi::size144px::Alarm, prelude::*};

pub struct AlarmRing {
    icon: Icon<Rgb565, Alarm>,
}

impl WatchApp for AlarmRing {
    fn new() -> Self {
        AlarmRing {
            icon: Alarm::new(Rgb565::GREEN),
        }
    }

    async fn on_start(&mut self, ctx: &mut AppContext) {
        ctx.vibration_pattern(Duration::from_millis(500), Duration::from_millis(500), 5)
            .await;
    }

    async fn on_stop(&mut self, _ctx: &mut AppContext) {}

    async fn on_event(&mut self, _event: SystemEvent, _ctx: &mut AppContext) -> EventResponse {
        EventResponse::Ignore
    }

    async fn render(&mut self, ctx: &mut AppContext) {
        let mut captures = ();
        let screen = Image::new(&self.icon)
            .flex_infinite_height(VerticalAlignment::Center)
            .flex_infinite_width(HorizontalAlignment::Center);
        let view = screen.as_drawable(Size::new(240, 240), Rgb565::BLACK, &mut captures);
        ctx.draw_screen(&view).await;
    }
}
