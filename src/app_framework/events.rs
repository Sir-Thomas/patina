use embedded_graphics::prelude::Point;

use crate::apps::AppId;

#[allow(unused)] // TODO: Remove these allows
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum SystemEvent {
    ButtonPress,
    Touch(TouchAction),
    ScreenTimeout,
    Tick,
}

#[allow(unused)] // TODO: Remove these allows
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum TouchAction {
    Down(Point),
    Up(Point),
    Swipe(Direction),
}

#[allow(unused)] // TODO: Remove these allows
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[allow(unused)] // TODO: Remove these allows
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum EventResponse {
    Rerender,
    SwitchApp(AppId),
    Ignore,
}