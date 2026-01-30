pub mod context;
pub mod traits;
pub mod events;

pub use context::AppContext;
pub use traits::WatchApp;
pub use events::{SystemEvent, EventResponse};

// Prelude for apps to use
#[allow(unused)] // TODO: Remove these allows
pub mod prelude {
    pub use crate::app_framework::{WatchApp, AppContext, SystemEvent, EventResponse};
    pub use embedded_graphics::{prelude::*, primitives::Rectangle, pixelcolor::Rgb565};
    pub use embedded_graphics::{mono_font::{MonoTextStyle, ascii::FONT_10X20}};
    pub use embedded_text::TextBox;
    pub use heapless::{String, Vec};
}


#[macro_export]
macro_rules! define_apps {
    ($($id:ident => $module:ident::$type:ty),* $(,)?) => {
        $(
            mod $module;
            use crate::apps::$module::*;
        )*

        #[allow(unused)] // TODO: Remove these allows
        #[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
        pub enum AppId {
            $($id,)*
        }
        
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