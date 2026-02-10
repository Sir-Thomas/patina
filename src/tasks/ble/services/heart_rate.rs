use trouble_host::prelude::*;

// Heart Rate service
// TODO: Replace dummy value
#[gatt_service(uuid = service::HEART_RATE)]
pub struct HeartRateService {
    // Heart Rate Measurement
    #[descriptor(uuid = descriptors::MEASUREMENT_DESCRIPTION, read, value = "Heart Rate")]
    #[characteristic(uuid = characteristic::HEART_RATE_MEASUREMENT, read, notify, value = 80)]
    level: u16,
}