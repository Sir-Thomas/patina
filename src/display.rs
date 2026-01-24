use defmt::{debug, info};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_nrf::{Peri, gpio::{Level, Output, OutputDrive}, peripherals, spim::{self, Spim}, spis};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{Delay, Timer};
use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::{PrimitiveStyleBuilder, Rectangle}};
use embedded_layout::align::{Align, horizontal, vertical};
use embedded_text::TextBox;
use heapless::String;
use lcd_async::{Builder, interface::SpiInterface, options::{self, Orientation, Rotation}, raw_framebuf::RawFrameBuf};
use static_cell::StaticCell;
use time::Duration;

use crate::{Irqs, time::CURRENT_TIME};

const WIDTH: usize = 240;
const HEIGHT: usize = 240;
const FRAMEBUFFER_ROWS: usize = 12;
const FRAMEBUFFER_HEIGHT: usize = HEIGHT / FRAMEBUFFER_ROWS;
const BYTES_PER_PIXEL: usize = 2;
const FRAMEBUFFER_SIZE: usize = WIDTH * FRAMEBUFFER_HEIGHT * BYTES_PER_PIXEL;
static FRAMEBUFFER: StaticCell<[u8; FRAMEBUFFER_SIZE]> = StaticCell::new();
static SPI_BUS: StaticCell<Mutex<NoopRawMutex, Spim<'static>>> = StaticCell::new();

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

    // info!("Initializing spi bus");
    let mut spim_config = spim::Config::default();
    spim_config.frequency = spim::Frequency::M8;
    spim_config.mode = spis::MODE_3;
    let spim = spim::Spim::new(twispi0, irqs, sck_pin, miso_pin, mosi_pin, spim_config);
    let spi_bus = Mutex::new(spim);
    let spi_bus = SPI_BUS.init(spi_bus);

    // info!("Initializing display");
    let display_reset = Output::new(display_reset_pin, Level::Low, OutputDrive::Standard);
    let display_cs = Output::new(display_chip_select_pin, Level::High, OutputDrive::Standard);
    let display_spi = SpiDevice::new(spi_bus, display_cs);

    let data_command = Output::new(data_command_pin, Level::Low, OutputDrive::Standard);
    let display_spi_interface = SpiInterface::new(display_spi, data_command);

    let mut display = Builder::new(lcd_async::models::ST7789, display_spi_interface)
        .display_size(WIDTH as u16, HEIGHT as u16)
        // .display_offset(0, 80)
        .invert_colors(options::ColorInversion::Inverted)
        .reset_pin(display_reset)
        .init(&mut Delay)
        .await
        .unwrap();
    display.set_orientation(Orientation::default().rotate(Rotation::Deg0)).await.unwrap();

    info!("Initializing frame buffer");
    let framebuffer = FRAMEBUFFER.init_with(|| [0; FRAMEBUFFER_SIZE]);

    info!("Getting time");
    Timer::after_millis(100).await;
    let mut previous_time = current_time_watcher.try_get().unwrap() - Duration::minutes(1);
    let display_area = Rectangle::new(Point::new(0, 0), Size::new(WIDTH as u32, HEIGHT as u32));

    info!("Initializing styles");
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

    loop {
        debug!("Updating Display");
        let current_time = current_time_watcher.try_get().unwrap();
        debug!("Current Time: {:02}:{:02}:{:02}", current_time.hour(), current_time.minute(), current_time.second());

        if current_time.minute() != previous_time.minute() {
            let colon_text = TextBox::new(
                ":",
                Rectangle::new(Point::zero(), Size::new(LG_SEGMENT_WIDTH, LG_DIGIT_HEIGHT + LG_DIGIT_SPACING)),
                lg_digit_style
            );
            let hour_str = num_to_string(current_time.hour());
            let hours_text = TextBox::new(
                hour_str.as_str(),
                Rectangle::new(Point::zero(), LG_SIZE),
                lg_digit_style,
            );
            let minute_str = num_to_string(current_time.minute());
            let minutes_text = TextBox::new(
                minute_str.as_str(),
                Rectangle::new(Point::zero(), LG_SIZE),
                lg_digit_style,
            );
            let second_str = num_to_string(current_time.second());
            let seconds_text = TextBox::new(
                second_str.as_str(),
                Rectangle::new(Point::zero(), SM_SIZE),
                sm_digit_style,
            );

            let positioned_colon = colon_text.align_to(&display_area, horizontal::Center, vertical::Center);
            let positioned_seconds = seconds_text.align_to(&display_area, horizontal::Right, vertical::Bottom);
            let positioned_minutes = minutes_text.align_to(&display_area, horizontal::Right, vertical::Center);
            let positioned_hours = hours_text.align_to(&display_area, horizontal::Left, vertical::Center);

            for i in 0..FRAMEBUFFER_ROWS {
                let mut fbuf = RawFrameBuf::<Rgb565, _>::new(framebuffer.as_mut_slice(), WIDTH, FRAMEBUFFER_HEIGHT);
                fbuf.clear(Rgb565::BLACK).unwrap();
                let mut fbuf = fbuf.translated(Point::new(0, -((FRAMEBUFFER_HEIGHT * i) as i32)));
                positioned_colon.draw(&mut fbuf).unwrap();
                positioned_seconds.draw(&mut fbuf).unwrap();
                positioned_minutes.draw(&mut fbuf).unwrap();
                positioned_hours.draw(&mut fbuf).unwrap();
                display.show_raw_data(
                    0, (FRAMEBUFFER_HEIGHT * i) as u16,
                    WIDTH as u16, FRAMEBUFFER_HEIGHT as u16,
                    framebuffer)
                    .await
                    .unwrap();
            }
            previous_time = current_time;
        } else if current_time.second() != previous_time.second() {
            let second_str = num_to_string(current_time.second());
            let seconds_text = TextBox::new(
                second_str.as_str(),
                Rectangle::new(Point::zero(), SM_SIZE),
                sm_digit_style,
            );

            let positioned_seconds = seconds_text.align_to(&display_area, horizontal::Right, vertical::Bottom);

            for i in 10..FRAMEBUFFER_ROWS { // TODO: fix hardcoded 10
                let mut fbuf = RawFrameBuf::<Rgb565, _>::new(framebuffer.as_mut_slice(), WIDTH, FRAMEBUFFER_HEIGHT);
                fbuf.clear(Rgb565::BLACK).unwrap();
                let mut fbuf = fbuf.translated(Point::new(0, -((FRAMEBUFFER_HEIGHT * i) as i32)));
                positioned_seconds.draw(&mut fbuf).unwrap();
                display.show_raw_data(
                    0, (FRAMEBUFFER_HEIGHT * i) as u16,
                    WIDTH as u16, FRAMEBUFFER_HEIGHT as u16,
                    framebuffer)
                    .await
                    .unwrap();
            }
            previous_time = current_time;
        }

        Timer::after_millis(50).await;
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