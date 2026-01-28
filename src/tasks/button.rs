use defmt::info;
use embassy_time::{Duration, WithTimeout};
use pinetime_bsp::button::Button;

use crate::{app_framework::SystemEvent, signals::EVENT_QUEUE};

const HOLD_DURATION_FOR_SYSTEM_RESET: Duration = Duration::from_secs(5);

#[embassy_executor::task]
pub async fn button_task(mut button: Button) {
    loop {
        button.wait_for_press().await;
        EVENT_QUEUE.send(SystemEvent::ButtonPress).await;
        info!("[Button Task] Button pressed");
        let result = button.wait_for_release().with_timeout(HOLD_DURATION_FOR_SYSTEM_RESET).await;
        if result.is_err() {
            info!("[Button Task] Button held for 5 seconds");
            info!("              Performing system reset");
            cortex_m::peripheral::SCB::sys_reset();
        }
    }
}