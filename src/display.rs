use core::{cell::RefCell, ops::Not};

use defmt::{debug, info};
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_nrf::{Peri, gpio::{Level, Output, OutputDrive}, peripherals, spim::{self, Spim}, spis};
use embassy_sync::blocking_mutex::{Mutex as BlockingMutex, raw::NoopRawMutex};
use embassy_time::{Delay, Duration, Timer};
use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::{PrimitiveStyleBuilder, Rectangle}};
use embedded_layout::align::{Align, horizontal, vertical};
use embedded_text::TextBox;
use heapless::String;
use mipidsi::{interface::SpiInterface, models::ST7789, options::{Orientation, Rotation}};
use static_cell::StaticCell;

use crate::{Irqs, state::BACKLIGHT_SIGNAL, time::CURRENT_TIME};


static DISPLAY_BUFFER: StaticCell<[u8; 512]> = StaticCell::new();
static SPI_BUS: StaticCell<BlockingMutex<NoopRawMutex, RefCell<Spim<'static>>>> = StaticCell::new();

const LG_DIGIT_HEIGHT: u32 = 120;
const LG_DIGIT_WIDTH: u32 = 45;
const LG_DIGIT_SPACING: u32 = 15;
const LG_SEGMENT_WIDTH: u32 = 10;
const SM_DIGIT_HEIGHT: u32 = 24;
const SM_DIGIT_WIDTH: u32 = 12;
const SM_DIGIT_SPACING: u32 = 4;
const SM_SEGMENT_WIDTH: u32 = 3;
const LG_SIZE: Size = Size::new(LG_DIGIT_WIDTH * 2 + LG_DIGIT_SPACING, LG_DIGIT_HEIGHT + LG_DIGIT_SPACING);
const SM_SIZE: Size = Size::new(SM_DIGIT_WIDTH * 2 + SM_DIGIT_SPACING, SM_DIGIT_HEIGHT + SM_DIGIT_SPACING);

#[embassy_executor::task]
pub async fn display_task(
    twispi0: Peri<'static, peripherals::TWISPI0>,
    irqs: Irqs,
    sck_pin: Peri<'static, peripherals::P0_02>,
    mosi_pin: Peri<'static, peripherals::P0_03>,
    miso_pin: Peri<'static, peripherals::P0_04>,
    data_command_pin: Peri<'static, peripherals::P0_18>,
    display_chip_select_pin: Peri<'static, peripherals::P0_25>,
    display_reset_pin: Peri<'static, peripherals::P0_26>,
) {
    let mut current_time_watcher = CURRENT_TIME.dyn_anon_receiver();

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

    let data_command = Output::new(data_command_pin, Level::Low, OutputDrive::Standard);
    let buffer = DISPLAY_BUFFER.init([0_u8; 512]);
    let display_spi_interface = SpiInterface::new(display_spi, data_command, &mut *buffer);

    let mut display = mipidsi::Builder::new(ST7789, display_spi_interface)
        .display_size(240, 240)
        // .display_offset(0, 80)
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .reset_pin(display_reset)
        .init(&mut Delay)
        .unwrap();
    display.set_orientation(Orientation::default().rotate(Rotation::Deg0)).unwrap();

    display.clear(Rgb565::BLACK).unwrap();


    Timer::after_millis(100).await;
    let mut previous_time = current_time_watcher.try_get().unwrap();
    let display_area = Rectangle::new(Point::new(0, 0), display.size());

    let lg_digit_style = eg_seven_segment::SevenSegmentStyleBuilder::new()
        .digit_size(Size::new(LG_DIGIT_WIDTH, LG_DIGIT_HEIGHT))
        .digit_spacing(LG_DIGIT_SPACING)
        .segment_width(LG_SEGMENT_WIDTH)
        .segment_color(Rgb565::GREEN)
        .build();
    let sm_digit_style = eg_seven_segment::SevenSegmentStyleBuilder::new()
        .digit_size(Size::new(SM_DIGIT_WIDTH, SM_DIGIT_HEIGHT))
        .digit_spacing(SM_DIGIT_SPACING)
        .segment_width(SM_SEGMENT_WIDTH)
        .segment_color(Rgb565::GREEN)
        .build();
    let bg_style = PrimitiveStyleBuilder::new()
        .fill_color(Rgb565::BLACK)
        .build();

    let colon_text = TextBox::new(
        ":",
        Rectangle::new(Point::zero(), Size::new(LG_SEGMENT_WIDTH, LG_DIGIT_HEIGHT + LG_DIGIT_SPACING)),
        lg_digit_style
    );
    let hour_str = num_to_string(previous_time.hour());
    let hours_text = TextBox::new(
        hour_str.as_str(),
        Rectangle::new(Point::zero(), LG_SIZE),
        lg_digit_style,
    );
    let minute_str = num_to_string(previous_time.minute());
    let minutes_text = TextBox::new(
        minute_str.as_str(),
        Rectangle::new(Point::zero(), LG_SIZE),
        lg_digit_style,
    );
    let second_str = num_to_string(previous_time.second());
    let seconds_text = TextBox::new(
        second_str.as_str(),
        Rectangle::new(Point::zero(), SM_SIZE),
        sm_digit_style,
    );
    let positioned_colon = colon_text.align_to(&display_area, horizontal::Center, vertical::Center);
    positioned_colon.draw(&mut display).unwrap();
    let positioned_seconds = seconds_text.align_to(&display_area, horizontal::Right, vertical::Bottom);
    positioned_seconds.draw(&mut display).unwrap();
    let positioned_minutes = minutes_text.align_to(&display_area, horizontal::Right, vertical::Center);
    positioned_minutes.draw(&mut display).unwrap();
    let positioned_hours = hours_text.align_to(&display_area, horizontal::Left, vertical::Center);
    positioned_hours.draw(&mut display).unwrap();
    
    loop {
        debug!("Updating Display");

        let current_time = current_time_watcher.try_get().unwrap();

        debug!("Current Time: {:02}:{:02}:{:02}", current_time.hour(), current_time.minute(), current_time.second());

        if current_time.hour() != previous_time.hour() {
            let hour_str = num_to_string(current_time.hour());
            let hours_text = TextBox::new(
                hour_str.as_str(),
                Rectangle::new(Point::new(20, 20), Size::new(LG_DIGIT_WIDTH * 2 + LG_DIGIT_SPACING, LG_DIGIT_HEIGHT + LG_DIGIT_SPACING)),
                lg_digit_style,
            );
            let background = Rectangle::new(Point::zero(), LG_SIZE).into_styled(bg_style);
            let positioned_background = background.align_to(&display_area, horizontal::Left, vertical::Center);
            positioned_background.draw(&mut display).unwrap();
            let positioned_hours = hours_text.align_to(&display_area, horizontal::Left, vertical::Center);
            positioned_hours.draw(&mut display).unwrap();
        }
        if current_time.minute() != previous_time.minute() {
            let minute_str = num_to_string(current_time.minute());
            let minutes_text = TextBox::new(
                minute_str.as_str(),
                Rectangle::new(Point::new(20, 20), Size::new(LG_DIGIT_WIDTH * 2 + LG_DIGIT_SPACING, LG_DIGIT_HEIGHT + LG_DIGIT_SPACING)),
                lg_digit_style,
            );
            let background = Rectangle::new(Point::zero(), LG_SIZE).into_styled(bg_style);
            let positioned_background = background.align_to(&display_area, horizontal::Right, vertical::Center);
            positioned_background.draw(&mut display).unwrap();
            let positioned_minutes = minutes_text.align_to(&display_area, horizontal::Right, vertical::Center);
            positioned_minutes.draw(&mut display).unwrap();
        }
        if current_time.second() != previous_time.second() {
            let second_str = num_to_string(current_time.second());
            let seconds_text = TextBox::new(
                second_str.as_str(),
                Rectangle::new(Point::new(20, 20), Size::new(SM_DIGIT_WIDTH * 2 + SM_DIGIT_SPACING, SM_DIGIT_HEIGHT + SM_DIGIT_SPACING)),
                sm_digit_style,
            );
            let background = Rectangle::new(Point::zero(), SM_SIZE).into_styled(bg_style);
            let positioned_background = background.align_to(&display_area, horizontal::Right, vertical::Bottom);
            positioned_background.draw(&mut display).unwrap();
            let positioned_seconds = seconds_text.align_to(&display_area, horizontal::Right, vertical::Bottom);
            positioned_seconds.draw(&mut display).unwrap();
        }

        previous_time = current_time;

        Timer::after(Duration::from_millis(50)).await;
    }
}

fn num_to_string(num: u8) -> String<8> {
    let tens = num / 10;
    let units = num % 10;
    let mut num_string = String::<8>::new();
    num_string.push(char::from_digit(tens as u32, 10).unwrap()).unwrap();
    num_string.push(char::from_digit(units as u32, 10).unwrap()).unwrap();
    num_string
}

struct BacklightController<'a> {
    low: Output<'a>,
    medium: Output<'a>,
    high: Output<'a>,
    brightness_level: BrightnessLevel,
}

impl<'a> BacklightController<'a> {
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
            BrightnessLevel::Low => self.low.set_low(),
            BrightnessLevel::Medium => self.medium.set_low(),
            BrightnessLevel::High => self.high.set_low(),
        }
    }

    fn backlight_off(&mut self) {
        self.low.set_high();
        self.medium.set_high();
        self.high.set_high();
    }

    fn backlight_on(&mut self) {
        self.set_brightness(self.brightness_level);
    }
}

#[derive(Clone, Copy, defmt::Format)]
pub enum BrightnessLevel {
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, defmt::Format)]
pub enum Backlight {
    Off,
    On,
}

impl Not for Backlight {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Backlight::Off => Backlight::On,
            Backlight::On => Backlight::Off,
        }
    }
}

#[embassy_executor::task]
pub async fn backlight_task(
    backlight_low_pin: Peri<'static, peripherals::P0_14>,
    backlight_med_pin: Peri<'static, peripherals::P0_22>,
    backlight_high_pin: Peri<'static, peripherals::P0_23>,
) {
    let mut backlight_controller = BacklightController::new(backlight_low_pin, backlight_med_pin, backlight_high_pin);
    loop {
        match BACKLIGHT_SIGNAL.wait().await {
            Backlight::Off => { backlight_controller.backlight_off(); info!("Backlight Off"); },
            Backlight::On => { backlight_controller.backlight_on(); info!("Backlight On"); },
        }
    }
}