use defmt::{debug, info};
use eg_seven_segment::SevenSegmentStyle;
use embedded_text::TextBox;
use time::{Duration, OffsetDateTime};
use embedded_layout::prelude::*;

use crate::app_framework::prelude::*;

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

pub struct ClockApp{
    current_time: OffsetDateTime,
    previous_time: OffsetDateTime,
    lg_digit_style: SevenSegmentStyle<Rgb565>,
    sm_digit_style: SevenSegmentStyle<Rgb565>,
    display_area: Rectangle,
}

impl WatchApp for ClockApp {
    fn new() -> Self {
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
        
        ClockApp {
            current_time: OffsetDateTime::UNIX_EPOCH,
            previous_time: OffsetDateTime::UNIX_EPOCH - Duration::MINUTE,
            lg_digit_style,
            sm_digit_style,
            display_area: Rectangle::new(
                Point::zero(),
                Size::new(240, 240),
            ),
        }
    }
    
    fn on_start(&mut self, ctx: &mut AppContext) {
        info!("[Clock App]Starting Clock App");
        self.current_time = ctx.get_time();
        debug!(
            "[Clock App]Current Time: {:02}:{:02}:{:02}",
            self.current_time.hour(),
            self.current_time.minute(),
            self.current_time.second()
        );
    }
    
    fn on_stop(&mut self, _ctx: &mut AppContext) {
        // Clean up clock app state
    }

    fn on_event(&mut self, event: SystemEvent, ctx: &mut AppContext) -> EventResponse {
        match event {
            SystemEvent::Tick => {
                self.current_time = ctx.get_time();
                debug!(
                    "[Clock App]Current Time: {:02}:{:02}:{:02}",
                    self.current_time.hour(),
                    self.current_time.minute(),
                    self.current_time.second()
                );
                EventResponse::Rerender
            },
            SystemEvent::ButtonPress => {
                ctx.turn_off_display();
                EventResponse::Ignore
            }
            _ => EventResponse::Ignore,
        }
    }

    async fn render(&mut self, ctx: &mut AppContext) {
        if self.current_time.minute() != self.previous_time.minute() {
            debug!("[Clock App] Updating Minutes");
            {
                let colon_text = TextBox::new(
                    ":",
                    Rectangle::new(Point::zero(), Size::new(LG_SEGMENT_WIDTH, LG_DIGIT_HEIGHT + LG_DIGIT_SPACING)),
                    self.lg_digit_style
                );
                let positioned_colon = colon_text.align_to(&self.display_area, horizontal::Center, vertical::Center);
                ctx.draw(&positioned_colon, positioned_colon.bounding_box(), Rgb565::BLACK).await;
            }
            {
                let hour_str = num_to_string(self.current_time.hour());
                let hours_text = TextBox::new(
                    hour_str.as_str(),
                    Rectangle::new(Point::zero(), LG_SIZE),
                    self.lg_digit_style,
                );
                let positioned_hours = hours_text.align_to(&self.display_area, horizontal::Left, vertical::Center);
                ctx.draw(&positioned_hours, positioned_hours.bounding_box(), Rgb565::BLACK).await;
            }
            {
                let minute_str = num_to_string(self.current_time.minute());
                let minutes_text = TextBox::new(
                    minute_str.as_str(),
                    Rectangle::new(Point::zero(), LG_SIZE),
                    self.lg_digit_style,
                );
                let positioned_minutes = minutes_text.align_to(&self.display_area, horizontal::Right, vertical::Center);
                ctx.draw(&positioned_minutes, positioned_minutes.bounding_box(), Rgb565::BLACK).await;
            }
            {
                let second_str = num_to_string(self.current_time.second());
                let seconds_text = TextBox::new(
                    second_str.as_str(),
                    Rectangle::new(Point::zero(), SM_SIZE),
                    self.sm_digit_style,
                );
                let positioned_seconds = seconds_text.align_to(&self.display_area, horizontal::Right, vertical::Bottom);
                ctx.draw(&positioned_seconds, positioned_seconds.bounding_box(), Rgb565::BLACK).await;
            }
            self.previous_time = self.current_time;
        } else if self.current_time.second() != self.previous_time.second() {
            debug!("[Clock App] Updating Seconds");
            let second_str = num_to_string(self.current_time.second());
            let seconds_text = TextBox::new(
                second_str.as_str(),
                Rectangle::new(Point::zero(), SM_SIZE),
                self.sm_digit_style,
            );
            let positioned_seconds = seconds_text.align_to(&self.display_area, horizontal::Right, vertical::Bottom);
            ctx.draw(&positioned_seconds, positioned_seconds.bounding_box(), Rgb565::BLACK).await;
            
            self.previous_time = self.current_time;
        }
    }
}

fn num_to_string(num: u8) -> String<2> {
    let tens = num / 10;
    let units = num % 10;
    let mut num_string = String::<2>::new();
    num_string.push(char::from_digit(tens as u32, 10).unwrap()).unwrap();
    num_string.push(char::from_digit(units as u32, 10).unwrap()).unwrap();
    num_string
}