use heapless::format;
use pinetime_bsp::touch::TouchEvent;

use crate::{app_framework::prelude::*, apps::{ALL_APPS, APP_COUNT, AppId}};

pub struct AppMenuApp {
    scroll_offset: usize,
    applist: [AppId; APP_COUNT - 3], // Exclude non-launchable apps
}

impl AppMenuApp {
    fn handle_touch(&mut self, touch_event: TouchEvent) -> EventResponse{
        match touch_event.gesture {
            TouchGesture::SwipeDown => {
                if self.scroll_offset == 0 {
                    return EventResponse::CloseApp;
                }
                self.scroll_offset -= 1;
                EventResponse::Rerender
            }
            TouchGesture::SwipeUp => {
                if self.scroll_offset * 4 + 4 >= self.applist.len() {
                    return EventResponse::Ignore;
                }
                self.scroll_offset += 1;
                EventResponse::Rerender
            }
            TouchGesture::Tap => {
                self.launch_app(touch_event.location.y)
            }
            _ => EventResponse::Ignore,
        }
    }

    fn launch_app(&self, y: i32) -> EventResponse {
        let index = self.scroll_offset * 4 + (y as usize / 60);
        if index < self.applist.len() {
            EventResponse::SwitchApp(self.applist[index])
        } else {
            EventResponse::Ignore
        }
    }

    async fn draw_app(&self, ctx: &mut AppContext, i: usize) {
        let index = self.scroll_offset * 4 + i;
        let mut string = String::<16>::new();
        if index < self.applist.len() {
            string = format!("{:?}", self.applist[index]).unwrap();
        }
        let style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
        let textbox = TextBox::new(
            string.as_str(),
            Rectangle::new(Point::new(0, i as i32 * 60), Size::new(240, 60)),
            style,
        );
        ctx.draw(&textbox, Rgb565::BLACK).await;
    }
}

impl WatchApp for AppMenuApp {
    fn new() -> Self {
        let mut applist: [AppId; APP_COUNT - 3] = [AppId::HelloWorld; APP_COUNT - 3];
        let mut i = 0;
        for app in ALL_APPS.iter() {
            if app.is_launchable() {
                applist[i] = *app;
                i += 1;
            }
        }
        AppMenuApp {
            scroll_offset: 0,
            applist,
        }
    }

    async fn on_start(&mut self, _ctx: &mut AppContext) {
        // Initialize app state
    }
    
    async fn on_stop(&mut self, _ctx: &mut AppContext) {
        // Clean up app state
    }

    async fn on_event(&mut self, event: SystemEvent, _ctx: &mut AppContext) -> EventResponse {
        match event {
            SystemEvent::Touch(touch_event) => {
                self.handle_touch(touch_event)
            }
            _ => EventResponse::Ignore
        }
    }

    async fn render(&mut self, ctx: &mut AppContext) {
        for i in 0..4 {
            self.draw_app(ctx, i).await;
        }
    }
}