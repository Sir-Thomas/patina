use defmt::info;
use embassy_nrf::{Peri, gpio::{Input, Level, Output, OutputDrive, Pull}, peripherals};
use embassy_time::Timer;


#[embassy_executor::task]
pub async fn button_task(
    button_input: Peri<'static, peripherals::P0_13>,
    button_enable: Peri<'static, peripherals::P0_15>
) {
    let _button_enable = Output::new(button_enable, Level::High, OutputDrive::Standard);
    let mut button = Input::new(button_input, Pull::None);
    loop {
        button.wait_for_high().await;
        Timer::after_millis(200).await;
        button.wait_for_low().await;
        info!("Button pressed, Rebooting PineTime");
        cortex_m::peripheral::SCB::sys_reset(); // TODO: send event
    }
}