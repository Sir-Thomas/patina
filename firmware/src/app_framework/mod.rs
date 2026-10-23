pub mod context;
pub mod events;
pub mod traits;

pub use context::AppContext;
pub use events::{EventResponse, SystemEvent};
pub use traits::WatchApp;

// Prelude for apps to use
#[allow(unused)] // TODO: Remove these allows
pub mod prelude {
    pub use crate::app_framework::{AppContext, EventResponse, SystemEvent, WatchApp};
    pub use core::fmt::Write as _;
    pub use embedded_graphics::mono_font::{MonoTextStyle, ascii::FONT_10X20};
    pub use embedded_graphics::{
        image::Image, pixelcolor::Rgb565, prelude::*, primitives::Rectangle,
    };
    pub use embedded_layout::prelude::*;
    pub use embedded_text::TextBox;
    pub use heapless::{String, Vec};
    pub use pinetime_bsp::touch::TouchGesture;
}

#[macro_export]
macro_rules! define_apps {
    ($($id:ident => $module:ident::$type:ty),* $(,)?) => {
        $(
            mod $module;
            use crate::apps::$module::*;
        )*

        #[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
        pub enum AppId {
            $($id,)*
        }

        impl AppId {
            pub fn is_launchable(&self) -> bool {
                !matches!(self, AppId::AppMenu | AppId::Clock | AppId::Settings)
            }
        }

        pub const APP_COUNT: usize = [$(stringify!($id)),*].len();

        pub const ALL_APPS: &[AppId] = &[$(AppId::$id),*];

        pub enum AppInstance {
            $($id($type),)*
        }

        impl AppInstance {
            pub fn new(id: AppId) -> Self {
                match id {
                    $(AppId::$id => AppInstance::$id(<$type>::new()),)*
                }
            }

            pub async fn on_start(&mut self, ctx: &mut AppContext) {
                match self {
                    $(AppInstance::$id(app) => app.on_start(ctx).await,)*
                }
            }

            pub async fn on_stop(&mut self, ctx: &mut AppContext) {
                match self {
                    $(AppInstance::$id(app) => app.on_stop(ctx).await,)*
                }
            }

            pub async fn on_event(&mut self, event: SystemEvent, ctx: &mut AppContext) -> EventResponse {
                match self {
                    $(AppInstance::$id(app) => app.on_event(event, ctx).await,)*
                }
            }

            pub async fn render(&mut self, ctx: &mut AppContext) {
                match self {
                    $(AppInstance::$id(app) => app.render(ctx).await,)*
                }
            }
        }
    };
}
