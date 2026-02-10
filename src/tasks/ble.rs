use defmt::info;
use embassy_executor::Spawner;
use nrf_sdc::{SoftdeviceController, mpsl::MultiprotocolServiceLayer};
use pinetime_bsp::ble::BleController;
use static_cell::StaticCell;
use trouble_host::{HostResources, prelude::*};
use crate::{
    app_framework::SystemEvent,
    signals::EVENT_QUEUE,
    tasks::ble::{clients::sync_time, services::{cts::update_time, prelude::*}},
};

mod clients;
mod services;

// We have to pretend to be InfiniTime to get companion apps to connect
// TODO: Find an app that can connect to a custom device name, or implement our own companion app
pub const NAME: &str = "InfiniTime";

const L2CAP_MTU: usize = 27;
const L2CAP_CHANNELS_MAX: usize = 2;
type BleResources = HostResources<DefaultPacketPool, L2CAP_CHANNELS_MAX, L2CAP_MTU>;
static RESOURCES: StaticCell<BleResources> = StaticCell::new();
static STACK: StaticCell<Stack<'static, SoftdeviceController, DefaultPacketPool>> = StaticCell::new();
static SERVER: StaticCell<PatinaGattServer<'static>> = StaticCell::new();

#[gatt_server]
pub struct PatinaGattServer {
    #[service]
    battery: BatteryService,
    #[service]
    cts: CurrentTimeService,
    #[service]
    device_information: DeviceInformationService,
    // TODO: Implement DFU Support
    // #[service]
    // dfu: InfinitimeDfuService,
    #[service]
    heart_rate: HeartRateService,
}

#[embassy_executor::task]
pub async fn ble_runner(bluetooth: BleController, spawner: Spawner) {
    let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);

    let resources = RESOURCES.init(BleResources::new());
    let stack = STACK.init(trouble_host::new(bluetooth.sdc, resources).set_random_address(address));

    let Host { mut peripheral, runner, .. } = stack.build();

    let gatt_server = PatinaGattServer::new_with_config(
        GapConfig::Peripheral(
            PeripheralConfig { name: NAME, appearance: &appearance::watch::SMARTWATCH }
        )
    ).unwrap();

    let server = SERVER.init(gatt_server);

    spawner.must_spawn(mpsl_task(bluetooth.mpsl));
    spawner.must_spawn(host_task(runner));
    loop {
        match advertise(&mut peripheral, &server, stack).await {
            Ok(conn) => connection_events(&conn, &server).await,
            Err(_) => info!("[ble] Error"),
        }
    }
}

#[embassy_executor::task]
async fn mpsl_task(mpsl: &'static MultiprotocolServiceLayer<'static>) -> ! {
    mpsl.run().await
}

#[embassy_executor::task]
async fn host_task(mut runner: Runner<'static, SoftdeviceController<'static>, DefaultPacketPool>) {
    runner.run().await.unwrap();
}

async fn advertise<'a, 'b, 'c, C: Controller>(
    peripheral: &mut Peripheral<'a, C, DefaultPacketPool>,
    server: &'b PatinaGattServer<'_>,
    stack: &'a Stack<'a, SoftdeviceController<'a>, DefaultPacketPool>,
) -> Result<GattConnection<'a, 'b, DefaultPacketPool>, BleHostError<C::Error>> {
    const GAP_ADV_LIMIT: usize = 31;
    let mut advertiser_data = [0; GAP_ADV_LIMIT];
    let advertiser_len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::ServiceUuids16(&[service::BATTERY.to_le_bytes()]),
            AdStructure::CompleteLocalName(NAME.as_bytes()),
        ],
        &mut advertiser_data[..],
    )?;
    let advertiser = peripheral.advertise(
        &Default::default(),
        Advertisement::ConnectableScannableUndirected {
            adv_data: &advertiser_data[0..advertiser_len],
            scan_data: &[],
        },
    ).await.unwrap();
    info!("[ble] Advertising");
    let conn = advertiser.accept().await?;
    sync_time(stack, conn.clone()).await;
    let conn = conn.with_attribute_server(server)?;
    info!("[ble] Connection Established");
    EVENT_QUEUE.send(SystemEvent::BluetoothConnected).await;
    Ok(conn)
}

async fn connection_events(
    connection: &GattConnection<'_, '_, DefaultPacketPool>,
    server: &'_ PatinaGattServer<'_>,
) {
    loop {
        match connection.next().await {
            GattConnectionEvent::Disconnected { reason } => {
                info!("[gatt] disconnected: {:?}", reason);
                EVENT_QUEUE.send(SystemEvent::BluetoothDisconnected).await;
                break;
            }
            GattConnectionEvent::Gatt { event } => {
                handle_gatt_event(event, server).await;
                // match event.accept() {
                //     Ok(reply) => reply.send().await,
                //     Err(e) => info!("[gatt] error proccessing request: {:?}", e),
                // }
            }
            _ => {}
        }
    }
}

async fn handle_gatt_event(
    event: GattEvent<'_, '_, DefaultPacketPool>,
    server: &'_ PatinaGattServer<'_>,
) {
    match event {
        GattEvent::Read(event) => {
            event.accept().unwrap();
            info!("[gatt] read request");
        }
        GattEvent::Write(event) => {
            handle_write_event(event, server).await;
        }
        _ => {
            info!("[gatt] other event");
        }
    }
}

async fn handle_write_event(
    event: WriteEvent<'_, '_, DefaultPacketPool>,
    server: &'_ PatinaGattServer<'_>,
) {
    if event.handle() == server.cts.current_time.handle {
        info!("[gatt] Write Event to Time Characteristic: {:?}", event.data());
        update_time(event.data().try_into().unwrap());
    }
}