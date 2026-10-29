//! Display module

#[cfg(feature = "defmt")]
use defmt::info;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_nrf::{Peri, gpio::{Level, Output, OutputDrive}, peripherals, spim::Spim};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::Delay;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use mipidsi::{Builder, Display, interface::SpiInterface, models::ST7789, options::{ColorInversion, Orientation, Rotation}};
use static_cell::StaticCell;

use crate::ScreenOrientation;

const WIDTH: usize = 240;
const HEIGHT: usize = 240;

static DISPLAY_BUFFER: StaticCell<[u8; 512]> = StaticCell::new();

/// Display Controller
pub struct DisplayController {
    /// Display instance
    display: Display<SpiInterface<'static, SpiDevice<'static, NoopRawMutex, Spim<'static>, Output<'static>>, Output<'static>>, ST7789, Output<'static>>,
}

impl DisplayController {
    pub(crate) fn new(
        display_spi: SpiDevice<'static, NoopRawMutex, Spim<'static>, Output<'static>>,
        display_dc_pin: Peri<'static, peripherals::P0_18>,
        display_reset_pin: Peri<'static, peripherals::P0_26>,
    ) -> Self {
        let display_dc = Output::new(display_dc_pin, Level::Low, OutputDrive::Standard);
        let buffer = DISPLAY_BUFFER.init([0_u8; 512]);
        let display_spi_interface = SpiInterface::new(display_spi, display_dc, buffer);
        let display_reset = Output::new(display_reset_pin, Level::Low, OutputDrive::Standard);

        let mut display = Builder::new(ST7789, display_spi_interface)
            .display_size(WIDTH as u16, HEIGHT as u16)
            // .display_offset(0, 80)
            .invert_colors(ColorInversion::Inverted)
            .reset_pin(display_reset)
            .init(&mut Delay)
            .unwrap();
        display.set_orientation(Orientation::default()).unwrap();

        return Self {
            display,
        };
    }

    /// Clear the display with the given color
    pub fn clear(&mut self, color: Rgb565) {
        self.display.clear(color).unwrap();
    }

    /// Draw a drawable object to the display
    pub fn draw<D: Drawable<Color = Rgb565>>(&mut self, drawable: &D) {
        drawable.draw(&mut self.display).unwrap();
    }

    /// Put the display to sleep
    pub fn sleep(&mut self) {
        self.display.sleep(&mut Delay).unwrap();
    }

    /// Wake the display from sleep
    pub fn wake(&mut self) {
        self.display.wake(&mut Delay).unwrap();
    }

    pub(crate) fn set_orientation(&mut self, orientation: ScreenOrientation) {
        match orientation {
            ScreenOrientation::Normal => {
                self.display.set_orientation(Orientation::default()).unwrap();
            }
            ScreenOrientation::Flipped => {
                self.display.set_orientation(Orientation::default().flip_vertical()).unwrap();
            }
        }
    }
}