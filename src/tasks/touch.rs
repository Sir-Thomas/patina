use pinetime_bsp::touch::TouchController;

use crate::{app_framework::SystemEvent, signals::EVENT_QUEUE};

#[embassy_executor::task]
pub async fn touch_task(mut touch: TouchController) {
    loop {
        let touch_event = touch.wait_for_touch().await;
        EVENT_QUEUE.send(SystemEvent::Touch(touch_event)).await;
    }

}