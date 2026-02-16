use embassy_futures::select::select;
use embassy_time::Timer;
use pinetime_bsp::battery::BatteryController;

use crate::signals::BATTERY;


#[embassy_executor::task]
pub async fn battery_task(mut battery: BatteryController) {
    let sender = BATTERY.sender();
    sender.send((battery.charge_level().await, battery.is_charging(), battery.millivolts().await));
    loop {
        select(
            Timer::after_secs(300),
            battery.wait_for_charge_state_change(),
        ).await;
        sender.send((battery.charge_level().await, battery.is_charging(), battery.millivolts().await));
    }
}