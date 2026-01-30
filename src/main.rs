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
    // MCUboot only - comment out this block for running on baremetal
    info!("Updating vector table offset");
    let mut p = cortex_m::Peripherals::take().unwrap();
    unsafe {
        p.SCB.invalidate_icache();
        p.SCB.vtor.write(0x8200);
    }

    info!("Initializing PineTime");
    let mut config = embassy_nrf::config::Config::default();
    config.lfclk_source = embassy_nrf::config::LfclkSource::ExternalXtal;
    config.gpiote_interrupt_priority = Priority::P2;
    config.time_interrupt_priority = Priority::P2;
    let board = PineTime::new(config).await;

    info!("Spawning systick task");
    spawner.must_spawn(tasks::systick::systick_task());

    info!("Spawning app manager task");
    spawner.must_spawn(tasks::app_manager::app_manager(spawner, board));
}