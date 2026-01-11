use core::cell::RefCell;

use defmt::{debug, info};
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_nrf::{Peri, gpio::{Level, Output, OutputDrive}, peripherals, spim::{self, Spim}, spis};
use embassy_sync::blocking_mutex::{Mutex as BlockingMutex, raw::NoopRawMutex};
use embassy_time::{Delay, Duration, Timer};
use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::Rectangle};
use embedded_layout::{align::{Align, horizontal, vertical}, layout::linear::LinearLayout, prelude::Chain};
use embedded_text::TextBox;
use mipidsi::{interface::SpiInterface, models::ST7789, options::{Orientation, Rotation}};
use static_cell::StaticCell;
use u8g2_fonts::{U8g2TextStyle, fonts};

use crate::{Irqs, button::ButtonAction};
use crate::button::BUTTON_SIGNAL;


static DISPLAY_BUFFER: StaticCell<[u8; 512]> = StaticCell::new();
static SPI_BUS: StaticCell<BlockingMutex<NoopRawMutex, RefCell<Spim<'static>>>> = StaticCell::new();


#[embassy_executor::task]
pub async fn display_task(
    twispi0: Peri<'static, peripherals::TWISPI0>,
    irqs: Irqs,
    sck_pin: Peri<'static, peripherals::P0_02>,
    mosi_pin: Peri<'static, peripherals::P0_03>,
    miso_pin: Peri<'static, peripherals::P0_04>,
    data_clock_pin: Peri<'static, peripherals::P0_18>,
    display_chip_select_pin: Peri<'static, peripherals::P0_25>,
    display_reset_pin: Peri<'static, peripherals::P0_26>,
) {

    info!("Initializing spi bus");
    let mut spim_config = spim::Config::default();
    spim_config.frequency = spim::Frequency::M8;
    spim_config.mode = spis::MODE_3;
    let spim = spim::Spim::new(twispi0, irqs, sck_pin, miso_pin, mosi_pin, spim_config);
    let spi_bus = SPI_BUS.init(BlockingMutex::new(RefCell::new(spim)));

    info!("Initializing display");
    let display_reset = Output::new(display_reset_pin, Level::Low, OutputDrive::Standard);
    let display_cs = Output::new(display_chip_select_pin, Level::High, OutputDrive::Standard);
    let display_spi = SpiDevice::new(spi_bus, display_cs);

    let data_clock = Output::new(data_clock_pin, Level::Low, OutputDrive::Standard);
    let buffer = DISPLAY_BUFFER.init([0_u8; 512]);
    let display_spi_interface = SpiInterface::new(display_spi, data_clock, &mut *buffer);

    let mut display = mipidsi::Builder::new(ST7789, display_spi_interface)
        .display_size(240, 240)
        // .display_offset(0, 80)
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .reset_pin(display_reset)
        .init(&mut Delay)
        .unwrap();
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

struct Backlight<'a> {
    low: Output<'a>,
    medium: Output<'a>,
    high: Output<'a>,
    brightness_level: BrightnessLevel,
}

impl<'a> Backlight<'a> {
    fn new(
        backlight_low_pin: Peri<'static, peripherals::P0_14>,
        backlight_med_pin: Peri<'static, peripherals::P0_22>,
        backlight_high_pin: Peri<'static, peripherals::P0_23>,
    ) -> Self {
        Self {
            low: Output::new(backlight_low_pin, Level::High, OutputDrive::Standard),
            medium: Output::new(backlight_med_pin, Level::Low, OutputDrive::Standard),
            high: Output::new(backlight_high_pin, Level::High, OutputDrive::Standard),
            brightness_level: BrightnessLevel::Medium,
        }
    }

    fn set_brightness(&mut self, level: BrightnessLevel) {
        self.low.set_high();
        self.medium.set_high();
        self.high.set_high();
        self.brightness_level = level;
        match level {
            BrightnessLevel::Off => {},
            BrightnessLevel::Low => self.low.set_low(),
            BrightnessLevel::Medium => self.medium.set_low(),
            BrightnessLevel::High => self.high.set_low(),
        }
    }
}

#[derive(Clone, Copy, defmt::Format)]
enum BrightnessLevel {
    Off,
    Low,
    Medium,
    High,
}

#[embassy_executor::task]
pub async fn backlight_task(
    backlight_low_pin: Peri<'static, peripherals::P0_14>,
    backlight_med_pin: Peri<'static, peripherals::P0_22>,
    backlight_high_pin: Peri<'static, peripherals::P0_23>,
) {
    let mut backlight = Backlight::new(backlight_low_pin, backlight_med_pin, backlight_high_pin);
    loop {
        match BUTTON_SIGNAL.wait().await {
            ButtonAction::Press => {
                backlight.set_brightness(match backlight.brightness_level {
                    BrightnessLevel::Off => BrightnessLevel::Low,
                    BrightnessLevel::Low => BrightnessLevel::Medium,
                    BrightnessLevel::Medium => BrightnessLevel::High,
                    BrightnessLevel::High => BrightnessLevel::Low,
                });
                info!("Brightness level set: {}", backlight.brightness_level);
            }
            ButtonAction::Release => {}
        }
    }
}