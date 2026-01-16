# Flash MCUBoot
```
probe-rs erase --chip nRF52832_xxAA
probe-rs download bootloader-1.0.1.bin --binary-format Binary --chip nRF52832_xxAA
```

# Create venv for MCUBoot Image Tool
```
python3 -m venv .venv
source .venv/bin/activate
pip install -r scripts/requirements.txt
```


# Build for MCUBoot and Flash
Must be using MCUBoot values for memory.x and VTOR.
```
cargo objcopy --release -- -O binary patina.bin
scripts/imgtool.py create --header-size 32 --align 4 --version 1.0.0 --slot-size 475136 --pad-header patina.bin patina-image.bin
probe-rs download --chip nRF52832_xxAA --base-address 0x8000 --binary-format Binary patina-image.bin
```
You must be in the python venv to run the image tool.
```
source .venv/bin/activate
```

# Debug while booting from MCUBoot
```
probe-rs attach --chip nRF52832_xxAA target/thumbv7em-none-eabihf/release/patina
```

# Generate zip for OTA flash
You must be in the python venv to run adafruit-nrfutil.
```
adafruit-nrfutil dfu genpkg --dev-type 0x0052 --application patina-image.bin patina-dfu.zip
```
