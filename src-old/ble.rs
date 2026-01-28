use defmt::{unwrap, info};
use embassy_executor::Spawner;
use embassy_nrf::rng::Rng;
use embassy_nrf::{Peri, mode, peripherals};
use nrf_sdc::mpsl::{MultiprotocolServiceLayer, Peripherals as MpslPeripherals, raw};
use nrf_sdc::{Builder, Mem, Peripherals as SdcPeripherals, SoftdeviceController};
use static_cell::StaticCell;
use trouble_host::prelude::*;

use crate::Irqs;

const NAME: &str = "Patina";

/// Size of L2CAP packets
pub const L2CAP_MTU: usize = 27;
pub const L2CAP_TXQ: u8 = 10;
pub const L2CAP_RXQ: u8 = 10;

const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 2; // Signal + att
type BleResources = HostResources<DefaultPacketPool, L2CAP_CHANNELS_MAX, L2CAP_MTU>;
static RESOURCES: StaticCell<BleResources> = StaticCell::new();
static STACK: StaticCell<Stack<'static, SoftdeviceController, DefaultPacketPool>> = StaticCell::new();

#[gatt_server]
pub struct PineTimeServer {
    // nrfdfu: NrfDfuService,
    // battery: BatteryService,
    // infdfu: InfinitimeDfuService,
    // uart: NrfUartService,
}

pub fn create_mpsl(
    rtc0: Peri<'static, peripherals::RTC0>,
    timer0: Peri<'static, peripherals::TIMER0>,
    temp: Peri<'static, peripherals::TEMP>,
    ppi_ch19: Peri<'static, peripherals::PPI_CH19>,
    ppi_ch30: Peri<'static, peripherals::PPI_CH30>,
    ppi_ch31: Peri<'static, peripherals::PPI_CH31>,
    irqs: Irqs,
) -> &'static MultiprotocolServiceLayer<'static> {
    let lfclk_cfg = raw::mpsl_clock_lfclk_cfg_t {
        source: raw::MPSL_CLOCK_LF_SRC_RC as u8,
        rc_ctiv: 16,
        rc_temp_ctiv: 2,
        accuracy_ppm: raw::MPSL_DEFAULT_CLOCK_ACCURACY_PPM as u16,
        skip_wait_lfclk_started: raw::MPSL_DEFAULT_SKIP_WAIT_LFCLK_STARTED != 0,
    };
    let mpsl_p = MpslPeripherals::new(
        rtc0,
        timer0,
        temp,
        ppi_ch19,
        ppi_ch30,
        ppi_ch31,
    );

    static MPSL: StaticCell<MultiprotocolServiceLayer> = StaticCell::new();
    let mpsl = MPSL.init(MultiprotocolServiceLayer::new(mpsl_p, irqs, lfclk_cfg).unwrap());
    
    return mpsl;
}


#[embassy_executor::task]
pub async fn mpsl_task(mpsl: &'static MultiprotocolServiceLayer<'static>) -> ! {
    mpsl.run().await
}


#[embassy_executor::task]
async fn ble_task(mut runner: Runner<'static, SoftdeviceController<'static>, DefaultPacketPool>) {
    unwrap!(runner.run().await);
}


#[embassy_executor::task]
async fn advertise_task(
    // stack: &'static Stack<'static, SoftdeviceController<'static>, DefaultPacketPool>,
    mut peripheral: Peripheral<'static, SoftdeviceController<'static>, DefaultPacketPool>,
    // server: &'static PineTimeServer<'static>,
    // mut dfu_config: DfuConfig<'static>,
    // battery: &'static Battery<'static>,
) {
    // const BAS: [u8; 2] = [0x0F, 0x18];
    // const DFU: [u8; 2] = [0x59, 0xFE];
    let mut advertiser_data = [0; 31];
    unwrap!(AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            // AdStructure::ServiceUuids16(&[Uuid::Uuid16(BAS), Uuid::Uuid16(DFU)]),
            AdStructure::CompleteLocalName(NAME.as_bytes()),
        ],
        &mut advertiser_data[..],
    ));
    loop {
        info!("[ble] advertising");
        let advertiser = unwrap!(
            peripheral
                .advertise(
                    &Default::default(),
                    Advertisement::ConnectableScannableUndirected {
                        adv_data: &advertiser_data[..],
                        scan_data: &[],
                    },
                )
                .await
        );
        match advertiser.accept().await {
            Ok(_conn) => info!("Advertising"),// process(stack, conn, server, &mut dfu_config, battery).await,
            Err(e) => {
                info!("Error advertising: {:?}", e);
            }
        }
    }
}

pub fn start(
    mpsl: &'static MultiprotocolServiceLayer<'static>,
    sdc_p: SdcPeripherals<'static>,
    rng: Peri<'static, peripherals::RNG>,
    irqs: Irqs,
    spawner: Spawner,
) {
    info!("Initializing RNG");
    static RNG: StaticCell<Rng<'static, mode::Async>> = StaticCell::new();
    let rng = RNG.init(Rng::new(rng, irqs));

    info!("Initializing SDC Memory");
    const SDC_MEM_SIZE: usize = 2040;
    static SDC_MEM: StaticCell<nrf_sdc::Mem<SDC_MEM_SIZE>> = StaticCell::new();
    let sdc_mem = SDC_MEM.init(Mem::new());

    info!("Building sdc");
    let sdc = Builder::new().unwrap()
        .support_adv()
        .support_peripheral()
        .peripheral_count(1).unwrap()
        .buffer_cfg(L2CAP_MTU as u16, L2CAP_MTU as u16, L2CAP_TXQ, L2CAP_RXQ).unwrap()
        .build(sdc_p, rng, mpsl, sdc_mem)
        .unwrap();
    info!("sdc build complete");

    let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);
    info!("Our address = {:?}", address);

    let resources = RESOURCES.init(BleResources::new());
    let stack = STACK.init(trouble_host::new(sdc, resources).set_random_address(address));

    let Host { peripheral, runner, .. } = stack.build();

    // let gatt = unwrap!(PineTimeServer::new_with_config(GapConfig::Peripheral(
    //     PeripheralConfig {
    //         name: NAME,
    //         appearance: &appearance::watch::SMARTWATCH,
    //     }
    // ),));
    // static SERVER: StaticCell<PineTimeServer<'static>> = StaticCell::new();
    // let server = SERVER.init(gatt);

    spawner.must_spawn(ble_task(runner));
    spawner.must_spawn(advertise_task(peripheral));//(stack, peripheral, server, dfu_config, battery));
}