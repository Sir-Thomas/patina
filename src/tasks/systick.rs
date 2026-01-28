use defmt::debug;
use embassy_time::{Duration, Timer};
use time::OffsetDateTime;

use crate::{app_framework::events::SystemEvent, signals::{CURRENT_TIME, EVENT_QUEUE}};

const SYSTICK_DURATION: Duration = Duration::from_millis(50);
const BOOT_TICKS: embassy_time::Instant = embassy_time::Instant::MIN;

#[embassy_executor::task]
pub async fn systick_task() {
    let sender = CURRENT_TIME.sender();
    let mut current_time;
    let boot_time = time::OffsetDateTime::UNIX_EPOCH;
    let mut watchdog = unsafe { embassy_nrf::wdt::WatchdogHandle::steal::<embassy_nrf::peripherals::WDT>(0) };

    loop {
        debug!("System Tick");
        EVENT_QUEUE.send(SystemEvent::Tick).await;
        watchdog.pet();
        current_time = update_time(boot_time);
        sender.send(current_time);
        Timer::after(SYSTICK_DURATION).await;
    }
}

fn update_time(boot_time: OffsetDateTime) -> OffsetDateTime{
    let elapsed = (embassy_time::Instant::now() - BOOT_TICKS).as_millis() as i64;
    let current_time = boot_time.saturating_add(time::Duration::milliseconds(elapsed));
    // For debugging - print the current time
    let (hours, minutes, seconds) = current_time.to_hms();
    let millis = current_time.millisecond();
    debug!("[Systick] TIME: {:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis);
    current_time
}