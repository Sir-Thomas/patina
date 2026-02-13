pub mod battery;
pub mod cts;
pub mod device_information;
// pub mod infinitime_dfu;
pub mod heart_rate;
pub mod nordic_dfu;

pub mod prelude {
    pub use crate::tasks::ble::services::{
        battery::BatteryService,
        cts::CurrentTimeService,
        device_information::DeviceInformationService,
        // infinitime_dfu::InfinitimeDfuService,
        heart_rate::HeartRateService,
        nordic_dfu::NordicDfuService,
    };
}