use defmt::debug;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_nrf::{gpio::Output, spim::Spim};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::{Duration, Timer};
use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::Rectangle};
use embedded_layout::{align::{Align, horizontal, vertical}, layout::linear::LinearLayout, prelude::Chain};
use embedded_text::TextBox;
use mipidsi::{Display, interface::SpiInterface, models::ST7789};
use u8g2_fonts::{U8g2TextStyle, fonts};


#[embassy_executor::task]
pub async fn display_task(mut display: Display<SpiInterface<'static, SpiDevice<'static, NoopRawMutex, Spim<'static>, Output<'static>>, Output<'static>>, ST7789, Output<'static>>) {
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