use defmt::debug;
use embassy_time::{Instant, WithTimeout};
use time::OffsetDateTime;

use crate::{app_framework::events::SystemEvent, signals::{ADJUST_TIME, CURRENT_TIME, EVENT_QUEUE}};

const SYSTICK_DURATION: embassy_time::Duration = embassy_time::Duration::from_millis(1000);

#[embassy_executor::task]
pub async fn systick_task() {
    let sender = CURRENT_TIME.sender();
    let mut current_time;
    let mut boot_time = OffsetDateTime::UNIX_EPOCH;
    let mut watchdog = unsafe { embassy_nrf::wdt::WatchdogHandle::steal::<embassy_nrf::peripherals::WDT>(0) };

    loop {
        debug!("System Tick");
        EVENT_QUEUE.send(SystemEvent::Tick).await;
        watchdog.pet();
        current_time = update_time(boot_time);
        sender.send(current_time);
        let result = ADJUST_TIME.wait().with_timeout(SYSTICK_DURATION).await;
        if let Some(new_time) = result.ok() {
            debug!("[Systick] Adjusting time to: {:?}", new_time);
            // TODO: This will panic after approximately 3 years of uptime due to i64 overflow
            boot_time = new_time - time::Duration::milliseconds(Instant::now().as_millis().try_into().unwrap());
        }
    }
}

fn update_time(boot_time: OffsetDateTime) -> OffsetDateTime{
    let elapsed = (Instant::now()).as_millis() as i64;
    let current_time = boot_time.saturating_add(time::Duration::milliseconds(elapsed));
    // For debugging - print the current time
    let (hours, minutes, seconds) = current_time.to_hms();
    let millis = current_time.millisecond();
    debug!("[Systick] TIME: {:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis);
    current_time
}