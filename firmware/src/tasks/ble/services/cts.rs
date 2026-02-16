use trouble_host::prelude::*;

use crate::{signals::ADJUST_TIME, tasks::ble::clients::parse_time};

#[gatt_service(uuid = service::CURRENT_TIME)]
pub struct CurrentTimeService {
    #[characteristic(uuid = characteristic::CURRENT_TIME, write)]
    pub current_time: [u8; 10],
}

pub fn update_time(time: [u8; 10]) {
    ADJUST_TIME.signal(parse_time(time).unwrap());
}