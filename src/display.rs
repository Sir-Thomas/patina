use defmt::{debug, info};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_nrf::{Peri, gpio::{Level, Output, OutputDrive}, peripherals, spim::{self, Spim}, spis};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{Delay, Duration, Timer};
use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::{Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle}};
use embedded_layout::align::{Align, horizontal, vertical};
use embedded_text::TextBox;
use heapless::String;
use lcd_async::{Builder, interface::SpiInterface, options::{self, Orientation, Rotation}, raw_framebuf::RawFrameBuf};
use static_cell::StaticCell;

use crate::{Irqs, time::CURRENT_TIME};


static FRAME_BUFFER_SIZE: usize = 240 * 20 * 2;
static FRAME_BUFFER: StaticCell<[u8; FRAME_BUFFER_SIZE]> = StaticCell::new();
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

    // SPI pins for ESP32-C3 (adjust these according to your wiring)
    // let sclk = peripherals.GPIO6; // SCL
    // let mosi = peripherals.GPIO7; // SDA
    // let res = peripherals.GPIO10; // RES (Reset)
    // let dc = peripherals.GPIO2; // DC (Data/Command)
    // let cs = peripherals.GPIO3; // CS (Chip Select)

    let mut display = Builder::new(lcd_async::models::ST7789, display_spi_interface)
        .display_size(240, 240)
        // .display_offset(0, 80)
        .invert_colors(options::ColorInversion::Inverted)
        .reset_pin(display_reset)
        .init(&mut Delay)
        .await
        .unwrap();
    display.set_orientation(Orientation::default().rotate(Rotation::Deg0)).await.unwrap();

    // display.clear(Rgb565::BLACK).unwrap();

    info!("Initializing frame buffer");
    let frame_buffer = FRAME_BUFFER.init_with(|| [0; FRAME_BUFFER_SIZE]);

    for i in 0..12 {
        let mut fbuf = RawFrameBuf::<Rgb565, _>::new(frame_buffer.as_mut_slice(), 240, 20);

        fbuf.clear(Rgb565::RED).unwrap();
        display
            .show_raw_data(0, 20 * i as u16, 240 as u16, 20 as u16, frame_buffer)
            .await
            .unwrap();
    }
    info!("Display complete");

    loop {
        Timer::after_secs(5).await;
    }


    Timer::after_millis(100).await;
    let mut previous_time = current_time_watcher.try_get().unwrap();
    let display_area = Rectangle::new(Point::new(0, 0), Size::new(240, 240));

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
    // positioned_colon.draw(&mut display).unwrap();
    let positioned_seconds = seconds_text.align_to(&display_area, horizontal::Right, vertical::Bottom);
    // positioned_seconds.draw(&mut display).unwrap();
    let positioned_minutes = minutes_text.align_to(&display_area, horizontal::Right, vertical::Center);
    // positioned_minutes.draw(&mut display).unwrap();
    let positioned_hours = hours_text.align_to(&display_area, horizontal::Left, vertical::Center);
    // positioned_hours.draw(&mut display).unwrap();
    
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
            // positioned_background.draw(&mut display).unwrap();
            let positioned_hours = hours_text.align_to(&display_area, horizontal::Left, vertical::Center);
            // positioned_hours.draw(&mut display).unwrap();
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
            // positioned_background.draw(&mut display).unwrap();
            let positioned_minutes = minutes_text.align_to(&display_area, horizontal::Right, vertical::Center);
            // positioned_minutes.draw(&mut display).unwrap();
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
            // positioned_background.draw(&mut display).unwrap();
            let positioned_seconds = seconds_text.align_to(&display_area, horizontal::Right, vertical::Bottom);
            // positioned_seconds.draw(&mut display).unwrap();
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