#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::interrupt::Priority;
use panic_probe as _;
use pinetime_bsp::PineTime;

mod app_framework;
mod apps;
mod signals;
mod tasks;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    #[cfg(not(feature = "baremetal"))]
    update_vtable();

    info!("[main] Initializing PineTime");
    let mut config = embassy_nrf::config::Config::default();
    config.lfclk_source = embassy_nrf::config::LfclkSource::ExternalXtal;
    config.gpiote_interrupt_priority = Priority::P2;
    config.time_interrupt_priority = Priority::P2;
    let board = PineTime::new(config).await;

    info!("[main] Spawning systick task");
    spawner.spawn(tasks::systick::systick_task().unwrap());

    info!("[main] Spawning app manager task");
    spawner.spawn(tasks::app_manager::app_manager(spawner, board).unwrap());
}

fn update_vtable() {
    info!("[main] Updating vector table offset");
    let mut p =
        cortex_m::Peripherals::take().expect("Cortex-M peripherals should always be available");
    unsafe {
        p.SCB.invalidate_icache();
        p.SCB.vtor.write(0x8200);
    }
}
