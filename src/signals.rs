use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel, signal::Signal, watch::Watch};
use time::OffsetDateTime;

use crate::app_framework::events::SystemEvent;

const TIME_WATCHERS: usize = 0;
pub static CURRENT_TIME: Watch<ThreadModeRawMutex, OffsetDateTime, TIME_WATCHERS> = Watch::new();
pub static ADJUST_TIME: Signal<ThreadModeRawMutex, OffsetDateTime> = Signal::new();

const EVENT_QUEUE_SIZE: usize = 5;
pub static EVENT_QUEUE: Channel<ThreadModeRawMutex, SystemEvent, EVENT_QUEUE_SIZE> = Channel::new();

pub static REFRESH_TIMEOUT: Signal<ThreadModeRawMutex, ()> = Signal::new();
pub static CHANGE_DISPLAY_STATE: Signal<ThreadModeRawMutex, bool> = Signal::new();