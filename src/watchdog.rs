use defmt::debug;
use embassy_time::{Duration, Timer};


#[embassy_executor::task]
pub async fn watchdog_task() {
    let mut handle = unsafe { embassy_nrf::wdt::WatchdogHandle::steal::<embassy_nrf::peripherals::WDT>(0) };
    loop {
        debug!("Petting watchdog");
        handle.pet();
        Timer::after(Duration::from_secs(4)).await;
    }
}