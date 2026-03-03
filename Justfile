import? 'Justfile.local'

default:
    @just --list

build:
    rm -f memory.x
    cp memory-mcuboot.x memory.x
    DEFMT_LOG=off cargo objcopy --target thumbv7em-none-eabi --package patina-firmware --release -- -O binary ./images/patina.bin
    scripts/imgtool.py create --header-size 512 --align 4 --version 1.0.0 --slot-size 475136 --pad-header ./images/patina.bin ./images/patina-image.bin
    adafruit-nrfutil dfu genpkg --dev-type 0x0052 --application ./images/patina-image.bin ./images/patina-dfu.zip

run-baremetal:
    rm -f memory.x
    cp memory-baremetal.x memory.x
    DEFMT_LOG=off cargo run --package patina-firmware --release --features "baremetal"

run-mcuboot:
    rm -f memory.x
    cp memory-mcuboot.x memory.x
    DEFMT_LOG=off cargo objcopy --target thumbv7em-none-eabi --package patina-firmware --release -- -O binary ./images/patina.bin
    scripts/imgtool.py create --header-size 512 --align 4 --version 1.0.0 --slot-size 475136 --pad-header ./images/patina.bin ./images/patina-image.bin
    probe-rs download --chip nrf52832_xxaa --binary-format Binary --base-address 0x8000 ./images/patina-image.bin

debug-baremetal:
    rm -f memory.x
    cp memory-baremetal.x memory.x
    cargo run --package patina-firmware --release --features "baremetal"

debug-mcuboot:
    rm -f memory.x
    cp memory-mcuboot.x memory.x
    cargo objcopy --package patina-firmware --release -- -O binary ./images/patina.bin
    scripts/imgtool.py create --header-size 512 --align 4 --version 1.0.0 --slot-size 475136 --pad-header ./images/patina.bin ./images/patina-image.bin
    probe-rs download --chip nrf52832_xxaa --binary-format Binary --base-address 0x8000 ./images/patina-image.bin
    probe-rs attach --chip nrf52832_xxaa ./target/thumbv7em-none-eabi/release/patina-firmware

flash-bootloader:
    probe-rs erase --chip nrf52832_xxaa
    probe-rs download --chip nrf52832_xxaa --binary-format Binary ./images/bootloader.bin

sim:
    cargo run --package patina-simulator
