use cst816s::CST816S; // TODO: Decouple from driver
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_nrf::{gpio::{Input, Output}, twim::Twim};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;

use crate::{app_framework::SystemEvent, signals::EVENT_QUEUE};

#[embassy_executor::task]
pub async fn touch_task(mut touch: CST816S<I2cDevice<'static, NoopRawMutex, Twim<'static>>, Input<'static>, Output<'static>>) {
    loop {
        let touch_event = touch.wait_for_touch().await.unwrap();
        EVENT_QUEUE.send(SystemEvent::Touch(touch_event)).await;
    }

}