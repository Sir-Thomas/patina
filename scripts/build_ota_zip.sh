#!/bin/sh

DEFMT_LOG=off cargo objcopy --release -- -O binary ./images/patina.bin
scripts/imgtool.py create --header-size 512 --align 4 --version 1.0.0 --slot-size 475136 --pad-header ./images/patina.bin ./images/patina-image.bin
adafruit-nrfutil dfu genpkg --dev-type 0x0052 --application ./images/patina-image.bin ./images/patina-dfu.zip