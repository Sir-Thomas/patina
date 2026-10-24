import? 'Justfile.local'

default:
    @just --list

# Build zip file for flashing over BLE
build:
    rm -f memory.x
    cp memory-mcuboot.x memory.x
    DEFMT_LOG=off cargo objcopy --target thumbv7em-none-eabi --package patina-firmware --release -- -O binary ./images/patina.bin
    scripts/imgtool.py create --header-size 512 --align 4 --version 1.0.0 --slot-size 475136 --pad-header ./images/patina.bin ./images/patina-image.bin
    adafruit-nrfutil dfu genpkg --dev-type 0x0052 --application ./images/patina-image.bin ./images/patina-dfu.zip

# Run the firmware over a debugger without using a bootloader
run-baremetal:
    rm -f memory.x
    cp memory-baremetal.x memory.x
    DEFMT_LOG=off cargo run --package patina-firmware --release --features "baremetal"

# Run the firmware over a debugger on a device with a bootloader
run-mcuboot:
    rm -f memory.x
    cp memory-mcuboot.x memory.x
    DEFMT_LOG=off cargo objcopy --target thumbv7em-none-eabi --package patina-firmware --release -- -O binary ./images/patina.bin
    scripts/imgtool.py create --header-size 512 --align 4 --version 1.0.0 --slot-size 475136 --pad-header ./images/patina.bin ./images/patina-image.bin
    probe-rs download --chip nrf52832_xxaa --binary-format Binary --base-address 0x8000 ./images/patina-image.bin

# Run the firmware with debug messages over a debugger without using a bootloader
debug-baremetal:
    rm -f memory.x
    cp memory-baremetal.x memory.x
    cargo run --package patina-firmware --release --features "baremetal"

# Run the firmware with debug messages over a debugger on a device with a bootloader
debug-mcuboot:
    rm -f memory.x
    cp memory-mcuboot.x memory.x
    cargo objcopy --package patina-firmware --release -- -O binary ./images/patina.bin
    scripts/imgtool.py create --header-size 512 --align 4 --version 1.0.0 --slot-size 475136 --pad-header ./images/patina.bin ./images/patina-image.bin
    probe-rs download --chip nrf52832_xxaa --binary-format Binary --base-address 0x8000 ./images/patina-image.bin
    probe-rs attach --chip nrf52832_xxaa ./target/thumbv7em-none-eabi/release/patina-firmware

# Flash the bootloader stored at ./images/bootloader.bin to the device
flash-bootloader:
    probe-rs erase --chip nrf52832_xxaa
    probe-rs download --chip nrf52832_xxaa --binary-format Hex ./images/bootloader.hex

# WIP - Run the firmware on a simulator
sim:
    cargo run --package patina-simulator
