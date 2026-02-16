use trouble_host::prelude::*;

#[gatt_service(uuid = service::DEVICE_INFORMATION)]
pub struct DeviceInformationService {
    #[characteristic(uuid = characteristic::MANUFACTURER_NAME_STRING, read, value = "PINE64")]
    manufacturer_name: &'static str,
    #[characteristic(uuid = characteristic::MODEL_NUMBER_STRING, read, value = "Pinetime")]
    model_number: &'static str,
    #[characteristic(uuid = characteristic::SERIAL_NUMBER_STRING, read, value = "0")]
    serial_number: &'static str,
    #[characteristic(uuid = characteristic::FIRMWARE_REVISION_STRING, read, value = env!("CARGO_PKG_VERSION"))]
    firmware_revision: &'static str,
    #[characteristic(uuid = characteristic::HARDWARE_REVISION_STRING, read, value = "1.0.0")]
    hardware_revision: &'static str,
    #[characteristic(uuid = characteristic::SOFTWARE_REVISION_STRING, read, value = env!("CARGO_PKG_NAME"))]
    software_revision: &'static str,
}