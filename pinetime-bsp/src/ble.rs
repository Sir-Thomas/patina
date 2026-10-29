//! BLE controller

use embassy_nrf::{Peri, mode::Async, peripherals, rng};
use nrf_sdc::mpsl::{MultiprotocolServiceLayer, Peripherals, SessionMem, raw};
use static_cell::StaticCell;

use crate::Irqs;

const L2CAP_MTU: usize = 27;
const L2CAP_TXQ: u8 = 10;
const L2CAP_RXQ: u8 = 10;

const SDC_MEM_SIZE: usize = 2112;

pub(crate) fn build_mpsl(
    irqs: Irqs,
    rtc0: Peri<'static, peripherals::RTC0>,
    timer0: Peri<'static, peripherals::TIMER0>,
    temp: Peri<'static, peripherals::TEMP>,
    ppi_ch19: Peri<'static, peripherals::PPI_CH19>,
    ppi_ch30: Peri<'static, peripherals::PPI_CH30>,
    ppi_ch31: Peri<'static, peripherals::PPI_CH31>,
    session_memory: &'static mut SessionMem<1>,
) -> Result<MultiprotocolServiceLayer<'static>, nrf_sdc::Error> {
    let mpsl_peripherals = Peripherals::new(rtc0, timer0, temp, ppi_ch19, ppi_ch30, ppi_ch31);
    let lfclk_cfg = raw::mpsl_clock_lfclk_cfg_t {
        source: raw::MPSL_CLOCK_LF_SRC_RC as u8,
        rc_ctiv: raw::MPSL_RECOMMENDED_RC_CTIV as u8,
        rc_temp_ctiv: raw::MPSL_RECOMMENDED_RC_TEMP_CTIV as u8,
        accuracy_ppm: raw::MPSL_DEFAULT_CLOCK_ACCURACY_PPM as u16,
        skip_wait_lfclk_started: raw::MPSL_DEFAULT_SKIP_WAIT_LFCLK_STARTED != 0,
    };
    MultiprotocolServiceLayer::with_timeslots(mpsl_peripherals, irqs, lfclk_cfg, session_memory)
}

pub(crate) fn build_sdc(
    p: nrf_sdc::Peripherals<'static>,
    rng: &'static mut rng::Rng<Async>,
    mpsl: &'static MultiprotocolServiceLayer<'static>,
) -> Result<nrf_sdc::SoftdeviceController<'static>, nrf_sdc::Error> {
    static SDC_MEM: StaticCell<nrf_sdc::Mem<SDC_MEM_SIZE>> = StaticCell::new();
    let sdc_mem = SDC_MEM.init(nrf_sdc::Mem::<SDC_MEM_SIZE>::new());
    nrf_sdc::Builder::new()?
        .support_adv()
        .support_peripheral()
        .peripheral_count(1)?
        .buffer_cfg(L2CAP_MTU as u16, L2CAP_MTU as u16, L2CAP_TXQ, L2CAP_RXQ)?
        .build(p, rng, mpsl, sdc_mem)
}

/// BLE Controller
pub struct BleController {
    /// Multiprotocol Service Layer
    pub mpsl: &'static MultiprotocolServiceLayer<'static>,
    /// SoftDevice Controller
    pub sdc: nrf_sdc::SoftdeviceController<'static>,
}

impl BleController {
    pub(crate) fn new(
        mpsl: &'static MultiprotocolServiceLayer<'static>,
        sdc: nrf_sdc::SoftdeviceController<'static>,
    ) -> Self {
        Self { mpsl, sdc }
    }
}
