#![no_std]
#![no_main]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
mod board;
pub use board::*;

pub mod backlight;
pub mod battery;
pub mod ble;
pub mod button;
pub mod display;
pub mod flash;
// pub mod motion;
pub mod touch;
pub mod vibrator;

// pub use embassy_nrf;
// pub use embassy_sync;
// pub use embassy_time;
// pub use lsm303agr; // need to switch to BMA425

// #[cfg(feature = "trouble")]
// pub mod ble;