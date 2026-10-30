//! Touch controller module

use cst816s_async::CST816S;
pub use cst816s_async::{Point, TouchEvent, TouchGesture};
#[cfg(feature = "defmt")]
use defmt::info;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_nrf::{
    gpio::{Input, Output},
    twim::Twim,
};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::Delay;

use crate::ScreenOrientation;

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
