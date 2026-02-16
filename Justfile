default:
    @just --list

build:
    DEFMT_LOG=off cargo objcopy --target thumbv7em-none-eabi --package patina-firmware --release -- -O binary ./images/patina.bin
    scripts/imgtool.py create --header-size 512 --align 4 --version 1.0.0 --slot-size 475136 --pad-header ./images/patina.bin ./images/patina-image.bin
    adafruit-nrfutil dfu genpkg --dev-type 0x0052 --application ./images/patina-image.bin ./images/patina-dfu.zip

run:
    DEFMT_LOG=off cargo objcopy --target thumbv7em-none-eabi --package patina-firmware --release -- -O binary ./images/patina.bin
    scripts/imgtool.py create --header-size 512 --align 4 --version 1.0.0 --slot-size 475136 --pad-header ./images/patina.bin ./images/patina-image.bin
    probe-rs download --chip nrf52832_xxaa --binary-format Binary --base-address 0x8000 ./images/patina-image.bin

debug:
    cargo objcopy --package patina-firmware --release -- -O binary ./images/patina.bin
    scripts/imgtool.py create --header-size 512 --align 4 --version 1.0.0 --slot-size 475136 --pad-header ./images/patina.bin ./images/patina-image.bin
    probe-rs download --chip nrf52832_xxaa --binary-format Binary --base-address 0x8000 ./images/patina-image.bin
    probe-rs attach --chip nrf52832_xxaa ./target/thumbv7em-none-eabi/release/patina-firmware

sim:
    cargo run --package patina-simulator