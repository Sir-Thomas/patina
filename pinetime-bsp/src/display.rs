//! Display module

#[cfg(feature = "defmt")]
use defmt::info;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_nrf::{Peri, gpio::{Level, Output, OutputDrive}, peripherals, spim::Spim};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::Delay;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::Rectangle};
use embedded_layout::View;
use lcd_async::{Builder, Display, interface::SpiInterface, models::ST7789, options::{ColorInversion, Orientation, Rotation}, raw_framebuf::RawFrameBuf};
use static_cell::StaticCell;

use crate::ScreenOrientation;

const WIDTH: usize = 240;
const HEIGHT: usize = 240;
const FRAMEBUFFER_ROWS: usize = 12;
const FRAMEBUFFER_HEIGHT: usize = HEIGHT / FRAMEBUFFER_ROWS;
const BYTES_PER_PIXEL: usize = 2;
const FRAMEBUFFER_SIZE: usize = WIDTH * FRAMEBUFFER_HEIGHT * BYTES_PER_PIXEL;
static FRAMEBUFFER: StaticCell<[u8; FRAMEBUFFER_SIZE]> = StaticCell::new();

/// Display Controller
pub struct DisplayController {
    /// Display instance
    display: Display<SpiInterface<SpiDevice<'static, NoopRawMutex, Spim<'static>, Output<'static>>, Output<'static>>, ST7789, Output<'static>>,
    /// Framebuffer
    framebuffer: &'static mut [u8; FRAMEBUFFER_SIZE],
}

impl DisplayController {
    pub(crate) async fn new(
        display_spi: SpiDevice<'static, NoopRawMutex, Spim<'static>, Output<'static>>,
        display_dc_pin: Peri<'static, peripherals::P0_18>,
        display_reset_pin: Peri<'static, peripherals::P0_26>,
    ) -> Self {
        let display_dc = Output::new(display_dc_pin, Level::Low, OutputDrive::Standard);
        let display_spi_interface = SpiInterface::new(display_spi, display_dc);
        let display_reset = Output::new(display_reset_pin, Level::Low, OutputDrive::Standard);

        #[cfg(feature = "defmt")]
        info!("Initializing display");
        let mut display = Builder::new(lcd_async::models::ST7789, display_spi_interface)
            .display_size(WIDTH as u16, HEIGHT as u16)
            // .display_offset(0, 80)
            .invert_colors(ColorInversion::Inverted)
            .reset_pin(display_reset)
            .init(&mut Delay)
            .await
            .unwrap();
        display.set_orientation(Orientation::default().rotate(Rotation::Deg0)).await.unwrap();

        #[cfg(feature = "defmt")]
        info!("Initializing frame buffer");
        let framebuffer = FRAMEBUFFER.init_with(|| [0; FRAMEBUFFER_SIZE]);
        return Self {
            display,
            framebuffer,
        };
    }

    /// Clear the display with the given color
    pub async fn clear(&mut self, color: Rgb565) {
        #[cfg(feature = "defmt")]
        info!("Clearing display");
        for i in 0..FRAMEBUFFER_ROWS {
            let mut fbuf = RawFrameBuf::<Rgb565, _>::new(self.framebuffer.as_mut_slice(), WIDTH, FRAMEBUFFER_HEIGHT);
            fbuf.clear(color).unwrap();
            self.display.show_raw_data(0, (FRAMEBUFFER_HEIGHT * i) as u16,
                WIDTH as u16, FRAMEBUFFER_HEIGHT as u16,
                self.framebuffer).await.unwrap();
        }
    }

    /// Draw a drawable object to the display
    pub async fn draw<D: Drawable<Color = Rgb565> + Dimensions>(&mut self, drawable: &D, background: Rgb565) {
        self.draw_impl(drawable, drawable.bounding_box(), background).await
    }

    /// Draw a drawable to the whole screen
    pub async fn draw_screen(&mut self, drawable: &impl Drawable<Color = Rgb565>) {
        for i in 0..FRAMEBUFFER_ROWS {
            let mut fbuf = RawFrameBuf::<Rgb565, _>::new(
                self.framebuffer.as_mut_slice(),
                WIDTH,
                FRAMEBUFFER_HEIGHT,
            );
            fbuf.clear(Rgb565::BLACK).unwrap();
            let mut fbuf = fbuf.translated(Point::new(0, -((FRAMEBUFFER_HEIGHT * i) as i32)));
            drawable.draw(&mut fbuf).unwrap();
            self.display.show_raw_data(
                0, (FRAMEBUFFER_HEIGHT * i) as u16,
                WIDTH as u16, FRAMEBUFFER_HEIGHT as u16,
                self.framebuffer
            ).await.unwrap();
        }
    }

    /// Draw a view to the display
    pub async fn draw_view<D: Drawable<Color = Rgb565> + View>(&mut self, drawable: &D, background: Rgb565) {
        let bounds = drawable.bounds();
        self.draw_impl(drawable, bounds, background).await
    }

    async fn draw_impl<D: Drawable<Color = Rgb565>>(&mut self, drawable: &D, bounds: Rectangle, background: Rgb565) {
        let start_x = bounds.top_left.x.max(0).min(WIDTH as i32);
        let end_x = bounds.bottom_right().unwrap().x.max(0).min((WIDTH - 1) as i32) + 1;
        let width = end_x.saturating_sub(start_x) as usize;
        let start_y = bounds.top_left.y.max(0).min(HEIGHT as i32);
        let end_y = bounds.bottom_right().unwrap().y.max(0).min((HEIGHT - 1) as i32) + 1;
        let height = end_y.saturating_sub(start_y) as usize;
        let fb_height = self.framebuffer.len() / (width * BYTES_PER_PIXEL);
        let rows = (height + fb_height - 1) / fb_height;

        if width == 0 || height == 0 {
            return;
        }

        for i in 0..rows {
            let y = start_y as usize + (fb_height * i);
            let y_max = y + fb_height;
            let height = y_max.min(end_y as usize) - y;
            let size = width * height * BYTES_PER_PIXEL;
            let mut fbuf = RawFrameBuf::<Rgb565, _>::new(
                self.framebuffer.as_mut_slice(),
                width,
                height,
            );
            fbuf.clear(background).unwrap();
            let mut fbuf = fbuf.translated(
                Point::new(
                    -start_x,
                    -(y as i32)
                )
            );
            drawable.draw(&mut fbuf).unwrap();
            self.display.show_raw_data(
                start_x as u16,
                y as u16,
                width as u16,
                height as u16,
                &self.framebuffer[..size])
                .await.unwrap();
        }
    }

    /// Put the display to sleep
    pub async fn sleep(&mut self) {
        self.display.sleep(&mut Delay).await.unwrap();
    }

    /// Wake the display from sleep
    pub async fn wake(&mut self) {
        self.display.wake(&mut Delay).await.unwrap();
    }

    /// Get screen orientation
    pub fn orientation(&self) -> ScreenOrientation {
        if self.display.orientation() == Orientation::default() {
            ScreenOrientation::Normal
        } else {
            ScreenOrientation::Flipped
        }
    }

    /// Set screen orientation
    pub async fn set_orientation(&mut self, orientation: ScreenOrientation) {
        info!("Setting display orientation");
        match orientation {
            ScreenOrientation::Normal => {
                self.display.set_orientation(Orientation::default().rotate(Rotation::Deg0)).await.unwrap();
            }
            ScreenOrientation::Flipped => {
                self.display.set_orientation(Orientation::default().rotate(Rotation::Deg180)).await.unwrap();
            }
        }
    }
}
