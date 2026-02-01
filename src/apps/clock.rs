use defmt::{debug, info};
use eg_seven_segment::SevenSegmentStyle;
use embedded_graphics::primitives::{Circle, PrimitiveStyle};
use embedded_text::TextBox;
use pinetime_bsp::touch::TouchGesture;
use time::{Duration, OffsetDateTime};
use embedded_layout::prelude::*;

use crate::{app_framework::prelude::*, apps::AppId};

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
    update_hours: bool,
    update_minutes: bool,
    update_seconds: bool,
    edit_time: bool,
    redraw_edit_indicator: bool,
}

impl ClockApp {
    fn adjust_time(&mut self, location: Point, ctx: &mut AppContext) {
        if location.x >= 190 && location.y >= 190 {
            self.update_seconds = true;
            ctx.reset_seconds();
        } else if location.x <= 120 && location.y <= 120 {
            self.update_hours = true;
            ctx.adjust_time(Duration::HOUR);
        } else if location.x <= 120 && location.y > 120 {
            self.update_hours = true;
            ctx.adjust_time(-Duration::HOUR);
        } else if location.x > 120 && location.y <= 120 {
            self.update_minutes = true;
            ctx.adjust_time(Duration::MINUTE);
        } else if location.x > 120 && location.y > 120 {
            self.update_minutes = true;
            ctx.adjust_time(-Duration::MINUTE);
        }
        self.current_time = ctx.time();
    }
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
            previous_time: OffsetDateTime::UNIX_EPOCH,
            lg_digit_style,
            sm_digit_style,
            display_area: Rectangle::new(
                Point::new(5, 5),
                Size::new(230, 230),
            ),
            update_hours: true,
            update_minutes: true,
            update_seconds: true,
            edit_time: false,
            redraw_edit_indicator: false,
        }
    }
    
    async fn on_start(&mut self, ctx: &mut AppContext) {
        info!("[Clock App]Starting Clock App");
        self.update_hours = true;
        self.update_minutes = true;
        self.update_seconds = true;
        self.current_time = ctx.time();
        debug!(
            "[Clock App]Current Time: {:02}:{:02}:{:02}",
            self.current_time.hour(),
            self.current_time.minute(),
            self.current_time.second()
        );
        let colon_text = TextBox::new(
            ":",
            Rectangle::new(Point::zero(), Size::new(LG_SEGMENT_WIDTH, LG_DIGIT_HEIGHT + LG_DIGIT_SPACING)),
            self.lg_digit_style
        );
        let positioned_colon = colon_text.align_to(&self.display_area, horizontal::Center, vertical::Center);
        ctx.draw(&positioned_colon, positioned_colon.bounding_box(), Rgb565::BLACK).await;
    }
    
    async fn on_stop(&mut self, _ctx: &mut AppContext) {
        self.edit_time = false;
        self.redraw_edit_indicator = true;
    }

    async fn on_event(&mut self, event: SystemEvent, ctx: &mut AppContext) -> EventResponse {
        match event {
            SystemEvent::Tick => {
                self.current_time = ctx.time();
                debug!(
                    "[Clock App]Current Time: {:02}:{:02}:{:02}",
                    self.current_time.hour(),
                    self.current_time.minute(),
                    self.current_time.second()
                );
                if self.current_time.hour() != self.previous_time.hour() {
                    self.update_hours = true;
                }
                if self.current_time.minute() != self.previous_time.minute() {
                    self.update_minutes = true;
                }
                if self.current_time.second() != self.previous_time.second() {
                    self.update_seconds = true;
                }
                if self.update_hours || self.update_minutes || self.update_seconds {
                    self.previous_time = self.current_time;
                    EventResponse::Rerender
                } else {
                    EventResponse::Ignore
                }
            },
            SystemEvent::Touch(event) => {
                match event.gesture {
                    TouchGesture::LongPress => {
                        self.redraw_edit_indicator = true;
                        self.edit_time = !self.edit_time;
                        ctx.short_vibration().await;
                        EventResponse::Rerender
                    }
                    TouchGesture::Tap => {
                        if !self.edit_time {
                            return EventResponse::Ignore;
                        }
                        self.adjust_time(event.location, ctx);
                        ctx.short_vibration().await;
                        EventResponse::Rerender
                    }
                    TouchGesture::SwipeDown => {
                        EventResponse::SwitchApp(AppId::Flashlight)
                    }
                    _ => EventResponse::Ignore,
                }
            },
            _ => EventResponse::Ignore,
        }
    }

    async fn render(&mut self, ctx: &mut AppContext) {
        info!("[Clock App] Rendering Clock");
        if self.update_hours {
            info!("[Clock App] Updating Hours");
            let hour_str = hour_to_string(self.current_time.hour());
            let hours_text = TextBox::new(
                hour_str.as_str(),
                Rectangle::new(Point::zero(), LG_SIZE),
                self.lg_digit_style,
            );
            let positioned_hours = hours_text.align_to(&self.display_area, horizontal::Left, vertical::Center);
            ctx.draw(&positioned_hours, positioned_hours.bounding_box(), Rgb565::BLACK).await;
            self.update_hours = false;
        }
        if self.update_minutes {
            info!("[Clock App] Updating Minutes");
            let minute_str = num_to_string(self.current_time.minute());
            let minutes_text = TextBox::new(
                minute_str.as_str(),
                Rectangle::new(Point::zero(), LG_SIZE),
                self.lg_digit_style,
            );
            let positioned_minutes = minutes_text.align_to(&self.display_area, horizontal::Right, vertical::Center);
            ctx.draw(&positioned_minutes, positioned_minutes.bounding_box(), Rgb565::BLACK).await;
            self.update_minutes = false;
        }
        if self.update_seconds {
            info!("[Clock App] Updating Seconds");
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
            self.update_seconds = false;
        }
        if self.redraw_edit_indicator {
            let circle = Circle::new(Point::zero(), 40)
                .into_styled(PrimitiveStyle::with_fill(if self.edit_time { Rgb565::GREEN } else { Rgb565::BLACK }))
                .align_to(&self.display_area, horizontal::Left, vertical::Bottom);
            ctx.draw(&circle, circle.bounding_box(), Rgb565::BLACK).await;
            self.redraw_edit_indicator = false;
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

fn hour_to_string(hour: u8) -> String<2> {
    match hour % 12 {
        0 => num_to_string(12),
        display_hour => num_to_string(display_hour),
    }
}