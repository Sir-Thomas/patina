#![no_std]
#![no_main]

use core::cell::RefCell;

use defmt::{debug, info, panic};
use defmt_rtt as _;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_nrf::{bind_interrupts, gpio::{AnyPin, Level, Output, OutputDrive}, interrupt::Priority, peripherals, spim::{self, Spim}, spis};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, blocking_mutex::Mutex as BlockingMutex};
use embassy_time::{Delay, Duration, Timer};
use embedded_graphics::{pixelcolor::Rgb565, prelude::{Point, Size, *}, primitives::Rectangle};
use embedded_layout::{align::{Align, horizontal, vertical}, layout::linear::LinearLayout, prelude::Chain};
use embedded_text::TextBox;
use mipidsi::{interface::SpiInterface, options::{Orientation, Rotation}};
use panic_probe as _;
use static_cell::StaticCell;
use u8g2_fonts::{U8g2TextStyle, fonts};

mod button;
mod display;
mod watchdog;

static DISPLAY_BUFFER: StaticCell<[u8; 512]> = StaticCell::new();
static SPI_BUS: StaticCell<BlockingMutex<NoopRawMutex, RefCell<Spim<'static>>>> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // MCUboot only - comment out this block for running on baremetal
    // info!("Updating vector table offset");
    // let mut p = cortex_m::Peripherals::take().unwrap();
    // unsafe {
    //     p.SCB.invalidate_icache();
    //     p.SCB.vtor.write(0x8100);
    // }

    info!("Binding interrupts");
    bind_interrupts!(struct Irqs {
        TWISPI0 => spim::InterruptHandler<peripherals::TWISPI0>;
        // TWISPI1 => twim::InterruptHandler<peripherals::TWISPI1>;
        // SAADC => saadc::InterruptHandler;
        // RNG => rng::InterruptHandler<RNG>;
        // EGU0_SWI0 => mpsl::LowPrioInterruptHandler;
        // CLOCK_POWER => mpsl::ClockInterruptHandler;
        // RADIO => mpsl::HighPrioInterruptHandler;
        // TIMER0 => mpsl::HighPrioInterruptHandler;
        // RTC0 => mpsl::HighPrioInterruptHandler;
    });

    info!("Initializing Embassy");
    let mut config = embassy_nrf::config::Config::default();
    config.lfclk_source = embassy_nrf::config::LfclkSource::InternalRC;
    config.gpiote_interrupt_priority = Priority::P2;
    config.time_interrupt_priority = Priority::P2;
    let p = embassy_nrf::init(config);

    info!("Spawning watchdog task");
    spawner.spawn(watchdog::watchdog_task()).unwrap();

    info!("Spawning button task");
    spawner.spawn(button::button_task(p.P0_13.into::<AnyPin>(), p.P0_15.into::<AnyPin>())).unwrap();

    info!("Initializing spi bus");
    let mut spim_config = spim::Config::default();
    spim_config.frequency = spim::Frequency::M8;
    spim_config.mode = spis::MODE_3;
    let spim = spim::Spim::new(p.TWISPI0, Irqs, p.P0_02, p.P0_04, p.P0_03, spim_config);
    let spi_bus = SPI_BUS.init(BlockingMutex::new(RefCell::new(spim)));

    info!("Initializing display");
    let _backlight_low = Output::new(p.P0_14, Level::High, OutputDrive::Standard);
    let _backlight_med = Output::new(p.P0_22, Level::High, OutputDrive::Standard);
    let _backlight_high = Output::new(p.P0_23, Level::Low, OutputDrive::Standard);
    let display_reset = Output::new(p.P0_26, Level::Low, OutputDrive::Standard);
    let display_cs = Output::new(p.P0_25, Level::High, OutputDrive::Standard);
    let display_spi = SpiDevice::new(spi_bus, display_cs);

    let data_clock = Output::new(p.P0_18, Level::Low, OutputDrive::Standard);
    let buffer = DISPLAY_BUFFER.init([0_u8; 512]);
    let display_spi_interface = SpiInterface::new(display_spi, data_clock, &mut *buffer);

    let mut display = mipidsi::Builder::new(mipidsi::models::ST7789, display_spi_interface)
        .display_size(240, 240)
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .reset_pin(display_reset)
        .init(&mut Delay)
        .unwrap_or_else(|_error| { panic!("Error initializing display"); });
    display.set_orientation(Orientation::default().rotate(Rotation::Deg0)).unwrap();

    info!("Clearing display");
    display.clear(Rgb565::BLACK).unwrap();

    info!("Spawning display task");
    spawner.spawn(display::display_task(display)).unwrap();

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}