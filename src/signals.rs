use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel, signal::Signal, watch::Watch};

use crate::app_framework::events::SystemEvent;

const TIME_WATCHERS: usize = 0;
pub static CURRENT_TIME: Watch<ThreadModeRawMutex, time::OffsetDateTime, TIME_WATCHERS> = Watch::new();

const EVENT_QUEUE_SIZE: usize = 5;
pub static EVENT_QUEUE: Channel<ThreadModeRawMutex, SystemEvent, EVENT_QUEUE_SIZE> = Channel::new();

pub static REFRESH_TIMEOUT: Signal<ThreadModeRawMutex, ()> = Signal::new();
pub static TIMEOUT_DISPLAY: Signal<ThreadModeRawMutex, bool> = Signal::new();