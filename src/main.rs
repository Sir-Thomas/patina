#![no_std]
#![no_main]

use core::cell::RefCell;

use defmt::{debug, info, panic};
use defmt_rtt as _;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_nrf::{Peri, bind_interrupts, gpio::{AnyPin, Input, Level, Output, OutputDrive, Pull}, interrupt::Priority, peripherals, spim::{self, Spim}, spis};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, blocking_mutex::Mutex as BlockingMutex};
use embassy_time::{Delay, Duration, Timer};
use embedded_graphics::{pixelcolor::Rgb565, prelude::{Point, Size, *}, primitives::Rectangle};
use embedded_layout::{align::{Align, horizontal, vertical}, layout::linear::LinearLayout, prelude::Chain};
use embedded_text::TextBox;
use mipidsi::{interface::SpiInterface, options::{Orientation, Rotation}};
use panic_probe as _;
use static_cell::StaticCell;
use u8g2_fonts::{U8g2TextStyle, fonts};

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

    let mut config = embassy_nrf::config::Config::default();
    config.lfclk_source = embassy_nrf::config::LfclkSource::InternalRC;
    config.gpiote_interrupt_priority = Priority::P2;
    config.time_interrupt_priority = Priority::P2;
    let p = embassy_nrf::init(config);

    info!("Spawning watchdog task");
    spawner.spawn(watchdog_task()).unwrap();

    info!("Spawning button task");
    let _button_enable = Output::new(p.P0_15, Level::High, OutputDrive::Standard);
    spawner.spawn(button_task(p.P0_13.into::<AnyPin>())).unwrap();

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
    let mut buffer = [0_u8; 256];
    let display_spi_interface = SpiInterface::new(display_spi, data_clock, &mut buffer);

    let mut display = mipidsi::Builder::new(mipidsi::models::ST7789, display_spi_interface)
        .display_size(240, 240)
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .reset_pin(display_reset)
        .init(&mut Delay)
        .unwrap_or_else(|_error| { panic!("Error initializing display"); });
    display.set_orientation(Orientation::default().rotate(Rotation::Deg0)).unwrap();

    info!("Clearing display");
    display.clear(Rgb565::BLACK).unwrap();
  
    loop {
        debug!("Updating display");
        let display_area = Rectangle::new(Point::zero(), display.size());

        let title = TextBox::new(
            "Patina",
            Rectangle::new(Point::zero(), Size::new(90, 12)),
            U8g2TextStyle::new(fonts::u8g2_font_6x10_tr, Rgb565::WHITE)
        );
        let feature = TextBox::new(
            "MCUBoot",
            Rectangle::new(Point::zero(), Size::new(90, 12)),
            U8g2TextStyle::new(fonts::u8g2_font_6x10_tr, Rgb565::WHITE)
        );
        const DIGIT_HEIGHT: u32 = 120;
        const DIGIT_WIDTH: u32 = 45;
        const DIGIT_SPACING: u32 = 15;
        const SEGMENT_WIDTH: u32 = 10;
        let total_width = 5 * DIGIT_WIDTH + 5 * DIGIT_SPACING;
        let clock = TextBox::new(
            "10:58",
            Rectangle::new(Point::zero(), Size::new(total_width, DIGIT_HEIGHT + DIGIT_SPACING)),
            eg_seven_segment::SevenSegmentStyleBuilder::new()
                .digit_size(Size::new(DIGIT_WIDTH, DIGIT_HEIGHT))
                .digit_spacing(DIGIT_SPACING)
                .segment_width(SEGMENT_WIDTH)
                .segment_color(Rgb565::GREEN)
                .build()
        );
        
        let header = LinearLayout::horizontal(
            Chain::new(title)
                .append(feature)
        )
            .with_alignment(vertical::Top)
            .arrange();

        let positioned_header = header.align_to(&display_area, horizontal::Center, vertical::Top);
        let positioned_clock = clock.align_to(&display_area, horizontal::Center, vertical::Center)
            .translate(Point::new(10, 0));

        positioned_header.draw(&mut display).unwrap();
        positioned_clock.draw(&mut display).unwrap();

        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
async fn watchdog_task() {
    let mut handle = unsafe { embassy_nrf::wdt::WatchdogHandle::steal::<embassy_nrf::peripherals::WDT>(0) };
    loop {
        debug!("Petting watchdog");
        handle.pet();
        Timer::after(Duration::from_secs(4)).await;
    }
}

#[embassy_executor::task]
async fn button_task(pin: Peri<'static, AnyPin>) {
    let mut button = Input::new(pin, Pull::None);
    loop {
        button.wait_for_high().await;
        Timer::after_millis(200).await;
        button.wait_for_low().await;
        info!("Button pressed, Rebooting PineTime");
        cortex_m::peripheral::SCB::sys_reset();
    }
}