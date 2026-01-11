use defmt::debug;
use embassy_nrf::{Peri, gpio::{Input, Level, Output, OutputDrive, Pull}, peripherals};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use embassy_time::Timer;

#[derive(Clone, Copy)]
pub enum ButtonAction {
    Press,
    Release,
}

pub static BUTTON_SIGNAL: Signal<ThreadModeRawMutex, ButtonAction> = Signal::new();


#[embassy_executor::task]
pub async fn button_task(
    button_input: Peri<'static, peripherals::P0_13>,
    button_enable: Peri<'static, peripherals::P0_15>
) {
    let _button_enable = Output::new(button_enable, Level::High, OutputDrive::Standard);
    let mut button = Input::new(button_input, Pull::None);
    loop {
        button.wait_for_high().await;
        debug!("Button pressed");
        BUTTON_SIGNAL.signal(ButtonAction::Press);
        Timer::after_millis(200).await;
        button.wait_for_low().await;
        debug!("Button released");
        BUTTON_SIGNAL.signal(ButtonAction::Release);
    }
}