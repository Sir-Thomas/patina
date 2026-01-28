use defmt::debug;
use embassy_futures::select::select;
use embassy_nrf::{Peri, gpio::{Input, Pull}, peripherals, saadc};
use embassy_sync::{blocking_mutex::raw::{NoopRawMutex, ThreadModeRawMutex}, mutex::Mutex, watch::Watch};
use embassy_time::Timer;

use crate::Irqs;

#[derive(Copy, Clone)]
pub struct Battery {
    percentage: u8,
    is_charging: bool,
}
static BATTERY_WATCHERS: usize = 0;
pub static BATTERY_LEVEL: Watch<ThreadModeRawMutex, Battery, BATTERY_WATCHERS> = Watch::new();

#[embassy_executor::task]
pub async fn battery_task(
    level_pin: Peri<'static, peripherals::P0_31>,
    saadc: Peri<'static, peripherals::SAADC>,
    irqs: Irqs,
    charge_pin: Peri<'static, peripherals::P0_12>,
) {
    let mut bat_config = saadc::ChannelConfig::single_ended(level_pin);
    bat_config.gain = saadc::Gain::GAIN1_4;
    bat_config.resistor = saadc::Resistor::BYPASS;
    bat_config.reference = saadc::Reference::INTERNAL;
    bat_config.time = saadc::Time::_40US;
    let mut adc_config = saadc::Config::default();
    adc_config.resolution = saadc::Resolution::_10BIT;
    let saadc = saadc::Saadc::new(saadc, irqs, adc_config, [bat_config]);
    let adc: Mutex<NoopRawMutex, saadc::Saadc<'static, 1>> = Mutex::new(saadc);
    
    let mut charging = Input::new(charge_pin, Pull::Up);
    let sender = BATTERY_LEVEL.sender();

    loop {
        let mut buf = [0i16; 1];
        let mut adc = adc.lock().await;
        adc.sample(&mut buf).await;
        let voltage = buf[0] as u32 * (8 * 600) / 1024;
        let percentage = approximate_charge(voltage);
        debug!("Battery voltage: {} mV, approx {}%, charging: {}", voltage, percentage, charging.is_low());
        let battery = Battery {
            percentage: percentage as u8,
            is_charging: charging.is_low(),
        };
        sender.send(battery);
        select(charging.wait_for_any_edge(), Timer::after_secs(300)).await;
    }
}

fn approximate_charge(voltage_millis: u32) -> u32 {
    let level_approx = &[(3500, 0), (3616, 3), (3723, 22), (3776, 48), (3979, 79), (4180, 100)];
    let approx = |value| {
        if value < level_approx[0].0 {
            level_approx[0].1
        } else {
            let mut ret = level_approx[level_approx.len() - 1].1;
            for i in 1..level_approx.len() {
                let prev = level_approx[i - 1];
                let val = level_approx[i];
                if value < val.0 {
                    ret = prev.1 + (value - prev.0) * (val.1 - prev.1) / (val.0 - prev.0);
                    break;
                }
            }
            ret
        }
    };
    approx(voltage_millis)
}