use pinetime_bsp::touch::TouchController;

use embassy_futures::select::{Either, select};

use crate::{
    app_framework::SystemEvent,
    signals::{EVENT_QUEUE, FLIP_SCREEN},
};

#[embassy_executor::task]
pub async fn touch_task(mut touch: TouchController) {
    loop {
        let event = select(touch.wait_for_touch(), FLIP_SCREEN.wait()).await;
        match event {
            Either::First(touch_event) => EVENT_QUEUE.send(SystemEvent::Touch(touch_event)).await,
            Either::Second(_) => touch.flip(),
        }
    }
}
