use embassy_time::WithTimeout;

use crate::{app_framework::SystemEvent, signals::{EVENT_QUEUE, REFRESH_TIMEOUT}};


#[embassy_executor::task]
pub async fn display_timeout_task() {
    const TIMEOUT_DURATION: embassy_time::Duration = embassy_time::Duration::from_secs(10);
    loop {
        let result = REFRESH_TIMEOUT.wait().with_timeout(TIMEOUT_DURATION).await;
        match result {
            Ok(_) => {},
            Err(_) => {
                EVENT_QUEUE.send(SystemEvent::ScreenTimeout).await;
                REFRESH_TIMEOUT.wait().await; 
            },
        }
    }
}