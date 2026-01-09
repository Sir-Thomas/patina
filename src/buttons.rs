#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_nrf::{Peri, gpio::{AnyPin, Input, Level, Output, OutputDrive, Pull}};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use embassy_time::{Duration, Timer, WithTimeout};
use panic_probe as _;

#[derive(Copy, Clone)]
enum PinetimeInput {
    Button,
    Screen
}

static SIGNAL: Signal<ThreadModeRawMutex, PinetimeInput> = Signal::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Starting...");
    let p = embassy_nrf::init(Default::default());
    spawner.spawn(adjustable_delay()).unwrap();
    let _button_enable = Output::new(p.P0_15, Level::High, OutputDrive::Standard);
    let _screen_enable = Output::new(p.P0_22, Level::High, OutputDrive::Standard);
    let button = input(p.P0_13.into::<AnyPin>(), "button pressed (fut)", PinetimeInput::Button);
    let screen = input(p.P0_28.into::<AnyPin>(), "screen tapped (fut)", PinetimeInput::Screen);
    join(button, screen).await;
}

async fn input(pin: Peri<'static, AnyPin>, message: &str, signal: PinetimeInput) {
    let mut button = Input::new(pin, Pull::None);
    loop {
        button.wait_for_high().await;
        info!("{}", message);
        SIGNAL.signal(signal);
        Timer::after_millis(200).await;
        button.wait_for_low().await;
    }
}

#[embassy_executor::task]
async fn adjustable_delay() {
    const INTERVAL_MS: u64 = 500;
    let mut delay_ms = 500;
    loop {
        info!("Delay = {} ms", delay_ms);
        let delay = Duration::from_millis(delay_ms);
        if let Some(v) = SIGNAL.wait().with_timeout(delay).await.ok() {
            delay_ms = match v {
                PinetimeInput::Screen => (delay_ms + INTERVAL_MS).min(2000),
                PinetimeInput::Button => (delay_ms - INTERVAL_MS).max(INTERVAL_MS),
            };
            info!("Adjusted delay to {} ms", delay_ms);
        }
    }
}