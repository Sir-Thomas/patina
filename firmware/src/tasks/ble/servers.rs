use crate::{signals::BATTERY, tasks::ble::PatinaGattServer};

#[embassy_executor::task]
pub async fn battery_task(server: &'static PatinaGattServer<'static>) {
    let mut receiver = BATTERY.receiver().unwrap();
    loop {
        let percentage = receiver.changed().await.0;
        server
            .set(&server.battery.battery_level, &percentage)
            .unwrap();

        // TODO: notification on low battery
    }
}
