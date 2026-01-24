#![no_std]
#![no_main]


use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::{bind_interrupts, interrupt::Priority, peripherals, rng, saadc, spim, twim};
use nrf_sdc::mpsl;
use panic_probe as _;

mod backlight;
mod battery;
mod ble;
mod button;
mod display;
mod state;
mod time;
mod touchscreen;
mod watchdog;

bind_interrupts!(struct Irqs {
    TWISPI0 => spim::InterruptHandler<peripherals::TWISPI0>;
    TWISPI1 => twim::InterruptHandler<peripherals::TWISPI1>;
    SAADC => saadc::InterruptHandler;
    RNG => rng::InterruptHandler<peripherals::RNG>;
    EGU0_SWI0 => mpsl::LowPrioInterruptHandler;
    CLOCK_POWER => mpsl::ClockInterruptHandler;
    RADIO => mpsl::HighPrioInterruptHandler;
    TIMER0 => mpsl::HighPrioInterruptHandler;
    RTC0 => mpsl::HighPrioInterruptHandler;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // MCUboot only - comment out this block for running on baremetal
    // info!("Updating vector table offset");
    // let mut p = cortex_m::Peripherals::take().unwrap();
    // unsafe {
    //     p.SCB.invalidate_icache();
    //     p.SCB.vtor.write(0x8100);
    // }

    info!("Initializing Embassy");
    let mut config = embassy_nrf::config::Config::default();
    config.lfclk_source = embassy_nrf::config::LfclkSource::InternalRC;
    config.gpiote_interrupt_priority = Priority::P2;
    config.time_interrupt_priority = Priority::P2;
    let p = embassy_nrf::init(config);

    info!("Spawning time task");
    spawner.spawn(time::time_task()).unwrap();

    info!("Spawning watchdog task");
    spawner.spawn(watchdog::watchdog_task()).unwrap();

    info!("Spawning battery task");
    spawner.spawn(battery::battery_task(
        p.P0_31,
        p.SAADC,
        Irqs,
        p.P0_12,
    )).unwrap();

    info!("Spawning button task");
    spawner.spawn(button::button_task(
        p.P0_13,
        p.P0_15
    )).unwrap();

    info!("Spawning backlight task");
    spawner.spawn(backlight::backlight_task(
        p.P0_14,
        p.P0_22,
        p.P0_23
    )).unwrap();

    info!("Spawning display task");
    spawner.spawn(display::display_task(
        p.TWISPI0,
        Irqs,
        p.P0_02,
        p.P0_03,
        p.P0_04,
        p.P0_18,
        p.P0_25,
        p.P0_26,
    )).unwrap();

    info!("Spawning touchscreen task");
    spawner.spawn(touchscreen::touchscreen_task(
        p.TWISPI1,
        Irqs,
        p.P0_06,
        p.P0_07,
        p.P0_11,
        p.P0_28,
        p.P0_10,
    )).unwrap();

    //TODO: Spawn BLE task
    info!("Creating mpsl");
    let mpsl = ble::create_mpsl(
        p.RTC0,
        p.TIMER0,
        p.TEMP,
        p.PPI_CH19,
        p.PPI_CH30,
        p.PPI_CH31,
        Irqs,
    );
    
    info!("Creating sdc peripherals");
    let sdc_p = nrf_sdc::Peripherals::new(
        p.PPI_CH17,
        p.PPI_CH18,
        p.PPI_CH20,
        p.PPI_CH21,
        p.PPI_CH22,
        p.PPI_CH23,
        p.PPI_CH24,
        p.PPI_CH25,
        p.PPI_CH26,
        p.PPI_CH27,
        p.PPI_CH28,
        p.PPI_CH29,
    );

    info!("Spawning BLE task");
    ble::start(mpsl, sdc_p, p.RNG, Irqs, spawner);
    //TODO: Spawn Accelerometer task
    //TODO: Spawn Heart Rate task
    //TODO: Spawn Vibration Motor task
    //TODO: Spawn Battery task
    
    // spawner.spawn(state::state_machine_task()).unwrap();
}