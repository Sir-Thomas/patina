use defmt::info;
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, with_timeout};
use nrf_sdc::SoftdeviceController;
use trouble_host::prelude::*;

use crate::signals::ADJUST_TIME;

// TODO: Transition to an embassy task that subscribes to notifications
pub async fn sync_time<'a>(
    stack: &'a Stack<'a, SoftdeviceController<'a>, DefaultPacketPool>,
    conn: Connection<'a, DefaultPacketPool>
) {
    info!("[ble] synchronizing time");
    let client = GattClient::<_, DefaultPacketPool, 10>::new(stack, &conn).await.unwrap();
    match select(
        client.task(),
        with_timeout(Duration::from_secs(8), async {
            let services = client.services_by_uuid(&Uuid::new_short(0x1805)).await?;
            for service in &services {
                info!("[ble] found service: {:?}", service);
            }
            if let Some(service) = services.first() {
                info!("[ble] found current time service");
                let c: Characteristic<u8> = client
                    .characteristic_by_uuid(&service, &Uuid::new_short(0x2a2b))
                    .await?;

                let mut data = [0; 10];
                client.read_characteristic(&c, &mut data[..]).await?;

                if let Some(time) = parse_time(data) {
                    let (h, m, s) = time.as_hms();
                    info!("[ble] received time: {:02}:{:02}:{:02}", h, m, s);
                    ADJUST_TIME.signal(time);
                }
            } else {
                info!("[ble] current time service not found");
            }
            Ok::<(), BleHostError<nrf_sdc::Error>>(())
        }),
    )
    .await
    {
        Either::First(_) => panic!("[ble] gatt client exited prematurely"),
        Either::Second(Ok(_)) => {
            info!("[ble] time sync completed");
        }
        Either::Second(Err(e)) => {
            info!("[ble] time sync error: {:?}", e);
        }
    }
}

pub fn parse_time(data: [u8; 10]) -> Option<time::PrimitiveDateTime> {
    let year = u16::from_le_bytes([data[0], data[1]]);
    let month = data[2];
    let day = data[3];
    let hour = data[4];
    let minute = data[5];
    let second = data[6];
    let _weekday = data[7];
    let secs_frac = data[8];

    if let Ok(month) = month.try_into() {
        let date = time::Date::from_calendar_date(year as i32, month, day);
        let micros = secs_frac as u32 * 1000000 / 256;
        let time = time::Time::from_hms_micro(hour, minute, second, micros);
        if let (Ok(time), Ok(date)) = (time, date) {
            let dt = time::PrimitiveDateTime::new(date, time);
            return Some(dt);
        }
    }
    info!("[ble] failed to parse time data: {:?}", data);
    None
}