use defmt::debug;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use embassy_time::Timer;

pub static SYSTICK_SIGNAL: Signal<ThreadModeRawMutex, ()> = Signal::new();

#[embassy_executor::task]
pub async fn watchdog_task() {
    let mut handle = unsafe { embassy_nrf::wdt::WatchdogHandle::steal::<embassy_nrf::peripherals::WDT>(0) };
    loop {
        debug!("Petting watchdog");
        SYSTICK_SIGNAL.signal(());
        handle.pet();
        Timer::after_millis(500).await;
    }
}