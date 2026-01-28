use core::cell::RefCell;

use cst816s::{CST816S, TouchEvent, TouchGesture};
use defmt::info;
use defmt_rtt as _;
use embassy_embedded_hal::shared_bus::blocking::i2c::I2cDevice;
use embassy_nrf::{Peri, gpio::{Input, Level, Output, OutputDrive, Pull}, peripherals, twim::{self, Twim}};
use embassy_sync::{blocking_mutex::{Mutex as BlockingMutex, raw::{NoopRawMutex, ThreadModeRawMutex}}, signal::Signal};
use embassy_time::Delay;
use panic_probe as _;
use static_cell::StaticCell;
use crate::Irqs;

static TOUCH_BUFFER: StaticCell<[u8; 256]> = StaticCell::new();
static I2C_BUS: StaticCell<BlockingMutex<NoopRawMutex, RefCell<Twim<'static>>>> = StaticCell::new();
pub static TOUCH_SIGNAL: Signal<ThreadModeRawMutex, TouchEvent> = Signal::new();

#[embassy_executor::task]
pub async fn touchscreen_task(
    twispi1: Peri<'static, peripherals::TWISPI1>,
    irqs: Irqs,
    sda: Peri<'static, peripherals::P0_06>,
    scl: Peri<'static, peripherals::P0_07>,
    // fake interrupt pin is used because driver has not implemented interrupts
    // we monitor the pin ourselves and wait for a signal
    fake_touchscreen_interrupt_pin: Peri<'static, peripherals::P0_11>,
    touchscreen_interrupt_pin: Peri<'static, peripherals::P0_28>,
    touchscreen_reset_pin: Peri<'static, peripherals::P0_10>,
) {
    let buffer = TOUCH_BUFFER.init([0; 256]);
    let mut twim_config = twim::Config::default();
    twim_config.frequency = twim::Frequency::K400;
    let i2c = Twim::new(twispi1, irqs, sda, scl, twim_config, buffer);
    let i2c_bus = I2C_BUS.init(BlockingMutex::new(RefCell::new(i2c)));

    let i2c_touch_device = I2cDevice::new(i2c_bus);
    let fake_touch_interrupt = Input::new(fake_touchscreen_interrupt_pin, Pull::Up); // see above
    let mut touch_interrupt = Input::new(touchscreen_interrupt_pin, Pull::Up); // TODO: why does this need mut
    let touch_reset = Output::new(touchscreen_reset_pin, Level::High, OutputDrive::Standard);
    let mut touch_controller = CST816S::new(i2c_touch_device, fake_touch_interrupt, touch_reset);
    if let Err(_e) = touch_controller.setup(&mut Delay) {
        defmt::panic!("Error initializing touch controller");
    }

    loop {
        touch_interrupt.wait_for_low().await;
        info!("TOUCHSCREEN: Touch event detected");
        if let Some(touch_event) = touch_controller.read_one_touch_event(false) {
            info!("Touch - x: {}, y: {}", touch_event.x, touch_event.y);
            match touch_event.action { // submit upstream patch?
                0 => info!("Action: Down"),
                1 => info!("Action: Up"),
                2 => info!("Action: Contact"),
                _ => info!("Action: Invalid touch action"),
            }
            match touch_event.gesture {
                TouchGesture::None => info!("Gesture: None"),
                TouchGesture::SlideUp => info!("Gesture: Slide Up"),
                TouchGesture::SlideDown => info!("Gesture: Slide Down"),
                TouchGesture::SlideLeft => info!("Gesture: Slide Left"),
                TouchGesture::SlideRight => info!("Gesture: Slide Right"),
                TouchGesture::SingleClick => info!("Gesture: Single Click"),
                TouchGesture::DoubleClick => info!("Gesture: Double Click"),
                TouchGesture::LongPress => info!("Gesture: Long Press"),
            }
            TOUCH_SIGNAL.signal(touch_event);
        }
    }
}