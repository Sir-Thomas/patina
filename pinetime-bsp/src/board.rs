use bma425::BMA425;
pub use bma425::PowerMode;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_nrf::config::Config;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pull};
use embassy_nrf::mode::Async;
use embassy_nrf::peripherals::{self, TWISPI0, TWISPI1};
use embassy_nrf::spim::Spim;
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{Peri, bind_interrupts, rng, saadc, spim, spis};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Duration;
use nrf_sdc::mpsl::{self, MultiprotocolServiceLayer};
use static_cell::StaticCell;

use crate::backlight::BacklightController;
pub use crate::backlight::BrightnessLevel;
use crate::battery::BatteryController;
use crate::ble::{BleController, build_mpsl, build_sdc};
use crate::button::Button;
use crate::display::DisplayController;
use crate::flash::FlashController;
use crate::heart_rate::HeartRateSensor;
use crate::touch::TouchController;
use crate::vibrator::Vibrator;

pub type Accelerometer = BMA425<I2cDevice<'static, NoopRawMutex, Twim<'static>>, Input<'static>>;

static I2C_BUFFER: StaticCell<[u8; 256]> = StaticCell::new();
static I2C_BUS: StaticCell<Mutex<NoopRawMutex, Twim<'static>>> = StaticCell::new();
static SPI_BUS: StaticCell<Mutex<NoopRawMutex, Spim<'static>>> = StaticCell::new();
static FLASH: StaticCell<Mutex<NoopRawMutex, mpsl::Flash<'static>>> = StaticCell::new();

/// Screen Orientation
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScreenOrientation {
    /// Default Screen Orientation
    Normal,
    /// Flipped Screen Orientation
    Flipped,
}

bind_interrupts!(
    pub(crate) struct Irqs {
        TWISPI0 => spim::InterruptHandler<TWISPI0>;
        TWISPI1 => twim::InterruptHandler<TWISPI1>;
        SAADC => saadc::InterruptHandler;
        RNG => rng::InterruptHandler<peripherals::RNG>;
        EGU0_SWI0 => mpsl::LowPrioInterruptHandler;
        CLOCK_POWER => mpsl::ClockInterruptHandler;
        RADIO => mpsl::HighPrioInterruptHandler;
        TIMER0 => mpsl::HighPrioInterruptHandler;
        RTC0 => mpsl::HighPrioInterruptHandler;
    }
);

/// Represents all the peripherals and pins available for the PineTime.
pub struct PineTime {
    /// Accelerometer
    pub accelerometer: Accelerometer,
    /// Backlight
    pub backlight: BacklightController,
    /// Battery
    pub battery: BatteryController,
    /// Bluetooth LE Controller
    pub bluetooth: BleController,
    /// Button
    pub button: Button,
    /// Display TODO: Improve this later
    pub display: DisplayController,
    /// Heart Rate Sensor
    pub heart_rate: HeartRateSensor,
    /// MPSL Flash
    pub mpsl_flash: &'static Mutex<NoopRawMutex, mpsl::Flash<'static>>,
    /// SPI Flash
    pub spi_flash: FlashController,
    /// Touch Screen
    pub touchscreen: TouchController,
    /// Vibrator
    pub vibrator: Vibrator,
    screen_orientation: ScreenOrientation,
}

impl PineTime {
    /// Create a new instance based on HAL configuration
    pub async fn new(config: Config) -> Self {
        let p = embassy_nrf::init(config);
        let i2c_bus = Self::initialize_i2c_bus(p.TWISPI1, p.P0_06, p.P0_07);
        let spi_bus = Self::initialize_spi_bus(p.TWISPI0, p.P0_02, p.P0_03, p.P0_04);
        static RNG: StaticCell<rng::Rng<Async>> = StaticCell::new();
        let rng = RNG.init(rng::Rng::new(p.RNG, Irqs));
        static MPSL: StaticCell<MultiprotocolServiceLayer> = StaticCell::new();
        static SESSION_MEMORY: StaticCell<mpsl::SessionMem<1>> = StaticCell::new();
        let mpsl = MPSL.init(
            build_mpsl(
                Irqs,
                p.RTC0,
                p.TIMER0,
                p.TEMP,
                p.PPI_CH19,
                p.PPI_CH30,
                p.PPI_CH31,
                SESSION_MEMORY.init(mpsl::SessionMem::new()),
            )
            .unwrap(),
        );
        let internal_flash = mpsl::Flash::take(mpsl, p.NVMC);
        let mpsl_flash = FLASH.init(Mutex::new(internal_flash));
        let sdc_peripherals = nrf_sdc::Peripherals::new(
            p.PPI_CH17, p.PPI_CH18, p.PPI_CH20, p.PPI_CH21, p.PPI_CH22, p.PPI_CH23, p.PPI_CH24,
            p.PPI_CH25, p.PPI_CH26, p.PPI_CH27, p.PPI_CH28, p.PPI_CH29,
        );
        let sdc = build_sdc(sdc_peripherals, rng, mpsl).unwrap();

        Self {
            accelerometer: BMA425::new(I2cDevice::new(&*i2c_bus), Input::new(p.P0_08, Pull::Down)),
            battery: BatteryController::new(p.P0_31, p.SAADC, Irqs, p.P0_12),
            backlight: BacklightController::new(p.P0_14, p.P0_22, p.P0_23),
            bluetooth: BleController::new(&*mpsl, sdc),
            button: Button::new(p.P0_15, p.P0_13, Duration::from_millis(100)),
            display: DisplayController::new(
                SpiDevice::new(
                    spi_bus,
                    Output::new(p.P0_25, Level::High, OutputDrive::Standard),
                ),
                p.P0_18,
                p.P0_26,
            )
            .await,
            heart_rate: HeartRateSensor::new(
                I2cDevice::new(&*i2c_bus),
                Input::new(p.P0_30, Pull::Up),
                Output::new(p.P0_29, Level::Low, OutputDrive::Standard),
            )
            .await,
            mpsl_flash,
            spi_flash: FlashController::new(SpiDevice::new(
                spi_bus,
                Output::new(p.P0_05, Level::High, OutputDrive::Standard),
            ))
            .await,
            touchscreen: TouchController::new(
                I2cDevice::new(&*i2c_bus),
                Input::new(p.P0_28, Pull::None),
                Output::new(p.P0_10, Level::High, OutputDrive::Standard),
                ScreenOrientation::Normal,
            )
            .await,
            vibrator: Vibrator::new(p.P0_16),
            screen_orientation: ScreenOrientation::Normal,
        }
    }

    fn initialize_i2c_bus(
        twispi1: Peri<'static, peripherals::TWISPI1>,
        p0_06: Peri<'static, peripherals::P0_06>,
        p0_07: Peri<'static, peripherals::P0_07>,
    ) -> &'static Mutex<NoopRawMutex, Twim<'static>> {
        let mut twim_config = twim::Config::default();
        twim_config.frequency = twim::Frequency::K400;
        let i2c_buffer = I2C_BUFFER.init([0; 256]);
        let i2c = Twim::new(twispi1, Irqs, p0_06, p0_07, twim_config, i2c_buffer);
        I2C_BUS.init(Mutex::new(i2c))
    }

    fn initialize_spi_bus(
        twispi0: Peri<'static, peripherals::TWISPI0>,
        sck_pin: Peri<'static, peripherals::P0_02>,
        mosi_pin: Peri<'static, peripherals::P0_03>,
        miso_pin: Peri<'static, peripherals::P0_04>,
    ) -> &'static Mutex<NoopRawMutex, Spim<'static>> {
        let mut spim_config = spim::Config::default();
        spim_config.frequency = spim::Frequency::M8;
        spim_config.mode = spis::MODE_3;
        let spim = spim::Spim::new(twispi0, Irqs, sck_pin, miso_pin, mosi_pin, spim_config);
        let spi_bus = Mutex::new(spim);
        SPI_BUS.init(spi_bus)
    }

    /// Set the screen orientation
    pub async fn set_screen_orientation(&mut self, orientation: ScreenOrientation) {
        self.screen_orientation = orientation;
        self.touchscreen.set_orientation(orientation);
        self.display.set_orientation(orientation).await;
    }
}
