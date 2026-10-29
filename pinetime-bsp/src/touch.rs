//! Touch controller module

use cst816s_async::CST816S;
#[cfg(feature = "defmt")]
use defmt::info;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_nrf::{
    gpio::{Input, Output},
    twim::Twim,
};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::Delay;
use embedded_graphics::prelude::Point;

use crate::ScreenOrientation;

/// Touch event data
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TouchEvent {
    /// Location
    pub location: Point,
    /// Gesture type
    pub gesture: TouchGesture,
}

impl From<cst816s_async::TouchEvent> for TouchEvent {
    fn from(event: cst816s_async::TouchEvent) -> Self {
        Self {
            location: Point::new(event.location.x as i32, event.location.y as i32),
            gesture: event.gesture.into(),
        }
    }
}

/// Touch gesture types
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TouchGesture {
    /// No gesture detected
    None,
    /// Swipe Down
    SwipeDown,
    /// Swipe up
    SwipeUp,
    /// Swipe left
    SwipeLeft,
    /// Swipe right
    SwipeRight,
    /// Tap
    Tap,
    /// Double Tap - This doesn't seem to ever be reported. The driver likely needs to be updated to enable double taps.
    DoubleTap,
    /// Long Press
    LongPress,
}

impl From<cst816s_async::TouchGesture> for TouchGesture {
    fn from(gesture: cst816s_async::TouchGesture) -> Self {
        match gesture {
            cst816s_async::TouchGesture::None => TouchGesture::None,
            cst816s_async::TouchGesture::SwipeDown => TouchGesture::SwipeDown,
            cst816s_async::TouchGesture::SwipeUp => TouchGesture::SwipeUp,
            cst816s_async::TouchGesture::SwipeLeft => TouchGesture::SwipeLeft,
            cst816s_async::TouchGesture::SwipeRight => TouchGesture::SwipeRight,
            cst816s_async::TouchGesture::Tap => TouchGesture::Tap,
            cst816s_async::TouchGesture::DoubleTap => TouchGesture::DoubleTap,
            cst816s_async::TouchGesture::LongPress => TouchGesture::LongPress,
        }
    }
}

/// Touch Controller
pub struct TouchController {
    controller:
        CST816S<I2cDevice<'static, NoopRawMutex, Twim<'static>>, Input<'static>, Output<'static>>,
    orientation: ScreenOrientation,
}

impl TouchController {
    /// Create a new TouchController
    pub async fn new(
        i2c_bus: I2cDevice<'static, NoopRawMutex, Twim<'static>>,
        interrupt_pin: Input<'static>,
        reset_pin: Output<'static>,
        orientation: ScreenOrientation,
    ) -> Self {
        let mut controller = CST816S::new(i2c_bus, interrupt_pin, reset_pin);
        controller.init(Delay).await.unwrap();
        Self {
            controller,
            orientation,
        }
    }

    /// Wait for a touch event
    pub async fn wait_for_touch(&mut self) -> TouchEvent {
        let mut event = self.controller.wait_for_touch().await.unwrap();
        if self.orientation == ScreenOrientation::Flipped {
            event.location.x = 240 - event.location.x;
            event.location.y = 240 - event.location.y;
            event.gesture = self.flip_gesture(event.gesture);
        }
        event.into()
    }

    pub(crate) fn set_orientation(&mut self, orientation: ScreenOrientation) {
        self.orientation = orientation;
    }

    /// Flip the touch input after initialization
    pub fn flip(&mut self) {
        info!("Flipping touch input");
        match self.orientation {
            ScreenOrientation::Normal => {
                self.set_orientation(ScreenOrientation::Flipped);
            }
            ScreenOrientation::Flipped => {
                self.set_orientation(ScreenOrientation::Normal);
            }
        }
    }

    fn flip_gesture(&self, gesture: cst816s_async::TouchGesture) -> cst816s_async::TouchGesture {
        match gesture {
            cst816s_async::TouchGesture::SwipeUp => cst816s_async::TouchGesture::SwipeDown,
            cst816s_async::TouchGesture::SwipeDown => cst816s_async::TouchGesture::SwipeUp,
            cst816s_async::TouchGesture::SwipeLeft => cst816s_async::TouchGesture::SwipeRight,
            cst816s_async::TouchGesture::SwipeRight => cst816s_async::TouchGesture::SwipeLeft,
            _ => gesture,
        }
    }
}
