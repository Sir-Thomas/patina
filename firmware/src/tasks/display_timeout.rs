use embassy_futures::select::{Either3, select3};
use embassy_time::{Duration, Timer};

use crate::{app_framework::SystemEvent, signals::{EVENT_QUEUE, REFRESH_TIMEOUT, CHANGE_DISPLAY_STATE}};

#[embassy_executor::task]
pub async fn display_timeout_task() {
    const TIMEOUT_DURATION: Duration = Duration::from_secs(10);
    let mut display_is_on = true;

    loop {
        if !display_is_on {
            display_is_on = CHANGE_DISPLAY_STATE.wait().await;
            continue;
        }

        let result = select3(
            REFRESH_TIMEOUT.wait(),
            CHANGE_DISPLAY_STATE.wait(),
            Timer::after(TIMEOUT_DURATION)
        ).await;
        match result {
            Either3::First(_) => {},
            Either3::Second(display_state) => {
                display_is_on = display_state;
            },
            Either3::Third(_) => {
                display_is_on = false;
                EVENT_QUEUE.send(SystemEvent::ScreenTimeout).await;
            },
        }
    }
}