use crate::app_framework::{AppContext, EventResponse, events::SystemEvent};

pub trait WatchApp {
    fn new() -> Self where Self: Sized;

    fn on_start(&mut self, ctx: &mut AppContext);
    
    fn on_stop(&mut self, ctx: &mut AppContext);

    fn on_event(&mut self, event: SystemEvent, ctx: &mut AppContext) -> EventResponse;

    async fn render(&mut self, ctx: &mut AppContext);
}