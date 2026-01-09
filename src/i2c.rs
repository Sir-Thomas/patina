#![no_std]
#![no_main]

use core::cell::RefCell;

use cst816s::CST816S;
use defmt::info;
use defmt_rtt as _;
use embassy_embedded_hal::shared_bus::blocking::i2c::I2cDevice;
use embassy_executor::Spawner;
use embassy_nrf::{bind_interrupts, gpio::{Input, Level, Output, OutputDrive, Pull}, peripherals::TWISPI1, twim::{self, Twim}};
use embassy_sync::{blocking_mutex::{raw::NoopRawMutex, Mutex as BlockingMutex}};
use embassy_time::{Delay, Timer};
use panic_probe as _;
use static_cell::StaticCell;

static I2C_BUS: StaticCell<BlockingMutex<NoopRawMutex, RefCell<Twim<'static>>>> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Starting...");
    let p = embassy_nrf::init(Default::default());
    bind_interrupts!(struct Irqs {
        TWISPI1 => twim::InterruptHandler<TWISPI1>;
    });

    let buffer = &mut [];
    let mut twim_config = twim::Config::default();
    twim_config.frequency = twim::Frequency::K400;
    let i2c = Twim::new(p.TWISPI1, Irqs, p.P0_06, p.P0_07, twim_config, buffer);
    let i2c_bus = I2C_BUS.init(BlockingMutex::new(RefCell::new(i2c)));

    let i2c_touch_device = I2cDevice::new(i2c_bus);
    let touch_interrupt = Input::new(p.P0_28, Pull::Up);
    let touch_reset = Output::new(p.P0_10, Level::High, OutputDrive::Standard);

    let mut touch_controller = CST816S::new(i2c_touch_device, touch_interrupt, touch_reset);
    if let Err(_e) = touch_controller.setup(&mut Delay) {
        defmt::panic!("Error initializing touch controller");
    }

    loop {
        if let Some(touch_event) = touch_controller.read_one_touch_event(false) {
            info!("Touch event");
            info!("x: {}, y: {}", touch_event.x, touch_event.y);
        }
        Timer::after_millis(200).await;
    }
}