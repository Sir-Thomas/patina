use pinetime_bsp::touch::TouchEvent;

use crate::apps::AppId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum SystemEvent {
    ButtonPress, // TODO: Deprecate. Button press never handled by app
    Touch(TouchEvent),
    ScreenTimeout, // TODO: Decide how timeout is handled.
    Tick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum EventResponse {
    CloseApp,
    Rerender,
    SwitchApp(AppId),
    Ignore,
}