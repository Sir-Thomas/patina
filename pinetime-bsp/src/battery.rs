//! Battery charge level and charging status
// TODO: test this on actual hardware
use embassy_nrf::{
    Peri,
    gpio::{Input, Pull},
    peripherals,
    saadc::{self, Saadc},
};

use crate::board::Irqs;

/// Battery controller
pub struct BatteryController {
    adc: Saadc<'static, 1>,
    charge_pin: Input<'static>,
}

impl BatteryController {
    /// Create a new battery controller
    pub(crate) fn new(
        level_pin: Peri<'static, peripherals::P0_31>,
        saadc: Peri<'static, peripherals::SAADC>,
        irqs: Irqs,
        charge_pin: Peri<'static, peripherals::P0_12>,
    ) -> BatteryController {
        let mut bat_config = saadc::ChannelConfig::single_ended(level_pin);
        bat_config.gain = saadc::Gain::Gain1_4;
        bat_config.resistor = saadc::Resistor::Bypass;
        bat_config.reference = saadc::Reference::Internal;
        bat_config.time = saadc::Time::_40US;
        let mut adc_config = saadc::Config::default();
        adc_config.resolution = saadc::Resolution::_10bit;
        let saadc = saadc::Saadc::new(saadc, irqs, adc_config, [bat_config]);
        BatteryController {
            adc: saadc,
            charge_pin: Input::new(charge_pin, Pull::Up),
        }
    }

    /// Get approximate battery charge level (0-100%)
    pub async fn charge_level(&mut self) -> u8 {
        let mv = self.millivolts().await;
        approximate_charge(mv)
    }

    /// Get battery voltage in millivolts
    pub async fn millivolts(&mut self) -> u32 {
        let mut buf = [0i16; 1];
        self.adc.sample(&mut buf).await;
        buf[0] as u32 * (8 * 600) / 1024
    }

    /// Returns true if the battery is currently charging
    pub fn is_charging(&self) -> bool {
        self.charge_pin.is_low()
    }

    /// Wait until the charging state changes and return the new state
    pub async fn wait_for_charge_state_change(&mut self) -> bool {
        self.charge_pin.wait_for_any_edge().await;
        self.is_charging()
    }
}

fn approximate_charge(voltage_millis: u32) -> u8 {
    let level_approx = &[
        (3500, 0),
        (3616, 3),
        (3723, 22),
        (3776, 48),
        (3979, 79),
        (4180, 100),
    ];
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
    approx(voltage_millis) as u8
}
