#![no_std]
#![no_main]

use core::cell::RefCell;

use defmt::info;
use defmt_rtt as _;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_nrf::{bind_interrupts, gpio::{Level, Output, OutputDrive}, peripherals, spim::{self, Spim}, spis};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, blocking_mutex::Mutex as BlockingMutex};
use embassy_time::{Delay, Timer};
use embedded_graphics::{mono_font::{MonoTextStyleBuilder, ascii::FONT_6X10}, pixelcolor::{BinaryColor, Rgb565}, prelude::{DrawTarget, *}, primitives::{Circle, PrimitiveStyle, Triangle}, text::{Baseline, Text}};
use mipidsi::{interface::SpiInterface, options::{Orientation, Rotation}};
use panic_probe as _;
use static_cell::StaticCell;

static SPI_BUS: StaticCell<BlockingMutex<NoopRawMutex, RefCell<Spim<'static>>>> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    bind_interrupts!(struct Irqs {
        TWISPI0 => spim::InterruptHandler<peripherals::TWISPI0>;
    });

    let p = embassy_nrf::init(Default::default());

    let mut spim_config = spim::Config::default();
    spim_config.frequency = spim::Frequency::M8;
    spim_config.mode = spis::MODE_3;
    let spim = spim::Spim::new(p.TWISPI0, Irqs, p.P0_02, p.P0_04, p.P0_03, spim_config);
    let spi_bus = SPI_BUS.init(BlockingMutex::new(RefCell::new(spim)));

    let _backlight_med = Output::new(p.P0_22, Level::Low, OutputDrive::Standard);
    let display_reset = Output::new(p.P0_26, Level::Low, OutputDrive::Standard);
    let display_cs = Output::new(p.P0_25, Level::High, OutputDrive::Standard);
    let display_spi = SpiDevice::new(spi_bus, display_cs);

    let data_clock = Output::new(p.P0_18, Level::Low, OutputDrive::Standard);
    let mut buffer = [0_u8; 512];
    let display_spi_interface = SpiInterface::new(display_spi, data_clock, &mut buffer);

    let mut display = mipidsi::Builder::new(mipidsi::models::ST7789, display_spi_interface)
        .display_size(240, 240)
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .reset_pin(display_reset)
        .init(&mut Delay)
        .unwrap();
    display.set_orientation(Orientation::default().rotate(Rotation::Deg0)).unwrap();

    info!("Display initialized");
    display.clear(Rgb565::BLACK).unwrap();
    info!("Display cleared");

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(Rgb565::WHITE)
        .background_color(Rgb565::BLACK)
        .build();

    Text::with_baseline("Hello, World!", Point::new(16, 16), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();
    info!("Text drawn to display");
    
    Timer::after_millis(500).await;

    draw_smiley(&mut display).unwrap();
    info!("Smiley drawn to display");

    loop {}
}

fn draw_smiley<T: DrawTarget<Color = Rgb565>>(display: &mut T) -> Result<(), T::Error> {
    // Draw the left eye as a circle located at (50, 50), with a diameter of 40, filled with white
    Circle::new(Point::new(50, 50), 40)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(display)?;

    // Draw the right eye as a circle located at (150, 50), with a diameter of 40, filled with white
    Circle::new(Point::new(150, 50), 40)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(display)?;

    // Draw an upside down red triangle to represent a smiling mouth
    Triangle::new(
        Point::new(90, 130),
        Point::new(150, 130),
        Point::new(120, 160),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
    .draw(display)?;

    // Cover the top part of the mouth with a black triangle so it looks closed instead of open
    Triangle::new(
        Point::new(100, 130),
        Point::new(140, 130),
        Point::new(120, 150),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
    .draw(display)?;

    Ok(())
}
