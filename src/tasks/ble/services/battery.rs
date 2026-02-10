use trouble_host::prelude::*;

// Battery service
// TODO: Replace dummy value
#[gatt_service(uuid = service::BATTERY)]
pub struct BatteryService {
    #[descriptor(uuid = descriptors::VALID_RANGE, read, value = [0, 100])]
    #[descriptor(uuid = descriptors::MEASUREMENT_DESCRIPTION, read, value = "Battery Level")]
    #[characteristic(uuid = characteristic::BATTERY_LEVEL, read, notify, value = 55)]
    battery_level: u8,
}