#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

/// Screen Orientation
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScreenOrientation {
    /// Default Screen Orientation
    Normal,
    /// Flipped Screen Orientation
    Flipped,
}

pub mod accelerometer;
pub mod backlight;
pub mod display;
pub mod vibrator;
