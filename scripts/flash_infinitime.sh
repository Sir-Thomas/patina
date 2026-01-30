#!/bin/sh

probe-rs erase --chip nrf52832_xxAA
probe-rs download --chip nRF52832_xxAA --binary-format Binary ./images/bootloader-1.0.1.bin 
probe-rs download --chip nRF52832_xxAA --binary-format Binary --base-address 0x8000 images/pinetime-mcuboot-app-image-1.16.0.bin