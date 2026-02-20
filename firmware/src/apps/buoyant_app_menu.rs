use buoyant::{if_view, primitives::Size, view::prelude::*};
use embedded_graphics::{mono_font::ascii::FONT_10X20, pixelcolor::Rgb565, prelude::RgbColor};
use heapless::{String, format};
use pinetime_bsp::touch::{TouchEvent, TouchGesture};

use crate::{app_framework::{AppContext, EventResponse, SystemEvent, WatchApp}, apps::{ALL_APPS, APP_COUNT, AppId}};

pub struct BuoyantAppMenuApp {
    scroll_offset: usize,
    applist: [AppId; APP_COUNT - 3], // Exclude non-launchable apps
}

impl BuoyantAppMenuApp {
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

    fn draw_app(&self, i: usize) -> impl View<Rgb565, ()> {
        let index = self.scroll_offset * 4 + i;
        let mut string = String::<16>::new();
        if index < self.applist.len() {
            string = format!("{:?}", self.applist[index]).unwrap();
        }
        if_view!((index < self.applist.len()) {
            ZStack::new((
                Capsule.foreground_color(Rgb565::GREEN),
                Text::new(string, &FONT_10X20).foreground_color(Rgb565::BLACK)
                    .padding(Edges::Leading, 30),
            )).with_horizontal_alignment(HorizontalAlignment::Leading)
        })
    }
}

impl WatchApp for BuoyantAppMenuApp {
    fn new() -> Self {
        let mut applist: [AppId; APP_COUNT - 3] = [AppId::HelloWorld; APP_COUNT - 3];
        let mut i = 0;
        for app in ALL_APPS.iter() {
            if app.is_launchable() {
                applist[i] = *app;
                i += 1;
            }
        }
        BuoyantAppMenuApp {
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
        let mut captures = ();
        let menu = VStack::new((
            self.draw_app(0),
            self.draw_app(1),
            self.draw_app(2),
            self.draw_app(3),
        )).with_spacing(10);
        let view = menu.as_drawable(Size::new(240,240), Rgb565::WHITE, &mut captures);
        ctx.draw(&view, Rgb565::BLACK).await;
    }
}