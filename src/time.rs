use defmt::debug;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, watch::Watch};

use crate::watchdog::SYSTICK_SIGNAL;

static TIME_WATCHERS: usize = 0;
pub static CURRENT_TIME: Watch<ThreadModeRawMutex, time::OffsetDateTime, TIME_WATCHERS> = Watch::new();

#[embassy_executor::task]
pub async fn time_task() {
    let boot_ticks = embassy_time::Instant::MIN;
    let boot_time = time::OffsetDateTime::UNIX_EPOCH;
    let mut current_time;
    let sender = CURRENT_TIME.sender();
    loop {
        SYSTICK_SIGNAL.wait().await;
        debug!("TIME: systick received");
        let elapsed = (embassy_time::Instant::now() - boot_ticks).as_millis() as i64;
        current_time = boot_time.saturating_add(time::Duration::milliseconds(elapsed));
        sender.send(current_time);
        let (hours, minutes, seconds) = current_time.to_hms();
        let millis = current_time.millisecond();
        debug!("TIME: {:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis);
    }
}