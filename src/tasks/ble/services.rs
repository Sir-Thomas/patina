pub mod battery;
pub mod cts;
pub mod device_information;
pub mod dfu;
pub mod heart_rate;

pub mod prelude {
    pub use crate::tasks::ble::services::{
        battery::BatteryService,
        cts::CurrentTimeService,
        device_information::DeviceInformationService,
        dfu::InfinitimeDfuService,
        heart_rate::HeartRateService,
    };
}