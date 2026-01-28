use embassy_futures::select::{Either3, select3};
use embassy_time::{Duration, Timer};

use crate::{app_framework::SystemEvent, signals::{EVENT_QUEUE, REFRESH_TIMEOUT, TIMEOUT_DISPLAY}};

#[embassy_executor::task]
pub async fn display_timeout_task() {
    const TIMEOUT_DURATION: Duration = Duration::from_secs(10);
    loop {
        let result = select3(REFRESH_TIMEOUT.wait(), TIMEOUT_DISPLAY.wait(), Timer::after(TIMEOUT_DURATION)).await;
        match result {
            Either3::First(_) => {},
            Either3::Second(enable_display) => {
                if !enable_display {
                    TIMEOUT_DISPLAY.wait().await;
                }
            },
            Either3::Third(_) => {
                EVENT_QUEUE.send(SystemEvent::ScreenTimeout).await;
            },
        }
    }
}