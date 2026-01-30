use cst816s::TouchEvent;

use crate::apps::AppId;

#[allow(unused)] // TODO: Remove these allows
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum SystemEvent {
    ButtonPress,
    Touch(TouchEvent),
    ScreenTimeout,
    Tick,
}

#[allow(unused)] // TODO: Remove these allows
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum EventResponse {
    CloseApp,
    Rerender,
    SwitchApp(AppId),
    Ignore,
}