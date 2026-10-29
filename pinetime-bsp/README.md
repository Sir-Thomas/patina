# pinetime-bsp

pinetime-bsp is a board support package (BSP) library for the BBC pine64 PineTime smartwatch.

This crate is heavily inspired by the excellent [microbit-bsp](https://crates.io/crates/microbit-bsp) crate.

## Features:

**Based on:**

* `embassy-nrf` HAL for peripherals
* Rust Async/Await

**Hardware Support:**

Should support the complete hardware features of the PineTime:

* Display
* Touch Input
* Button
* Vibration Motor
* Accelerometer
* Heart Rate Sensor
* Flash Storage
* Bluetooth LE support via `trouble-host` or `nrf-softdevice`

## TODO: Adjust everything below here for PineTime

## Example application

```rust
#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};

use microbit_bsp::*;

use {
    embassy_executor::Spawner,
    embassy_futures::select::{select, Either},
    embassy_time::Duration,
};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let board = Microbit::default();

    let mut display = board.display;
    let mut btn_a = board.btn_a;
    let mut btn_b = board.btn_b;

    display.set_brightness(display::Brightness::MAX);
    display.scroll("Hello, World!").await;
    defmt::info!("Application started, press buttons!");
    loop {
        match select(btn_a.wait_for_low(), btn_b.wait_for_low()).await {
            Either::First(_) => {
                display
                    .display(display::fonts::ARROW_LEFT, Duration::from_secs(1))
                    .await;
            }
            Either::Second(_) => {
                display
                    .display(display::fonts::ARROW_RIGHT, Duration::from_secs(1))
                    .await;
            }
        }
    }
}
```

## Examples

To run an example:

```bash
cd examples/display
cargo run --release
```

## Cargo Features

* `defmt` - enabled by default, and allows some crates to print things
* `trouble` - enables BLE support via the `trouble-host` crate