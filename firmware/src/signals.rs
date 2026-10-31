use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex, channel::Channel, signal::Signal, watch::Watch,
};
use time::PrimitiveDateTime;

use crate::app_framework::events::SystemEvent;

const TIME_WATCHERS: usize = 0;
pub static CURRENT_TIME: Watch<ThreadModeRawMutex, PrimitiveDateTime, TIME_WATCHERS> = Watch::new();
pub static ADJUST_TIME: Signal<ThreadModeRawMutex, PrimitiveDateTime> = Signal::new();
const ALARM_WATCHERS: usize = 1;
pub static SET_ALARM: Watch<ThreadModeRawMutex, Option<PrimitiveDateTime>, ALARM_WATCHERS> =
    Watch::new();

const BATTERY_WATCHERS: usize = 1;
pub static BATTERY: Watch<ThreadModeRawMutex, (u8, bool, u32), BATTERY_WATCHERS> = Watch::new();

const EVENT_QUEUE_SIZE: usize = 5;
pub static EVENT_QUEUE: Channel<ThreadModeRawMutex, SystemEvent, EVENT_QUEUE_SIZE> = Channel::new();

pub static REFRESH_TIMEOUT: Signal<ThreadModeRawMutex, ()> = Signal::new();
pub static CHANGE_DISPLAY_STATE: Signal<ThreadModeRawMutex, bool> = Signal::new();

pub static FLIP_SCREEN: Signal<ThreadModeRawMutex, ()> = Signal::new();
