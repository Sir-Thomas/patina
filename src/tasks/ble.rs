#![allow(dead_code)]
// Static services like DeviceInformationService trigger unused code warnings

use defmt::{debug, info};
use embassy_embedded_hal::{flash::partition::Partition, shared_bus::asynch::spi::SpiDevice};
use embassy_executor::Spawner;
use embassy_nrf::{gpio::Output, spim::Spim};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{Delay, Timer};
use heapless::Vec;
use nrf_dfu_target::prelude::*;
use nrf_sdc::{SoftdeviceController, mpsl::MultiprotocolServiceLayer};
use pinetime_bsp::{ble::BleController, flash::XT25F32B};
use spi_memory_async::series25::Flash;
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
pub const NAME: &str = "Patina";

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
    // TODO: add heart rate service
    // #[service]
    // heart_rate: HeartRateService,
    #[service]
    nordic_dfu: NordicDfuService,
}

struct DfuDevice<'a> {
    flash: Partition<'a, NoopRawMutex, Flash<SpiDevice<'static, NoopRawMutex, Spim<'static>, Output<'static>>, XT25F32B, Delay>>,
    target: DfuTarget<256>,
}

#[embassy_executor::task]
pub async fn ble_runner(bluetooth: BleController, flash: Flash<SpiDevice<'static, NoopRawMutex, Spim<'static>, Output<'static>>, XT25F32B, Delay>, spawner: Spawner) {
    let address: Address = Address::random([0xff, 0x8f, 0xaa, 0x05, 0xe4, 0xff]);

    let ficr = embassy_nrf::pac::FICR;
    let part = ficr.info().part().read().part().to_bits();
    let variant = ficr.info().variant().read().variant().to_bits();

    let hw_info = HardwareInfo {
        part,
        variant,
        rom_size: 0,
        ram_size: 0,
        rom_page_size: 0,
    };

    let fw_info = FirmwareInfo {
        ftype: FirmwareType::Application,
        version: 1,
        addr: 0,
        len: 0,
    };

    let flash_mutex: Mutex<NoopRawMutex, Flash<SpiDevice<'static, NoopRawMutex, Spim<'static>, Output<'static>>, XT25F32B, Delay>> = Mutex::new(flash);
    let dfu_partition = Partition::new(&flash_mutex, 0x00040000, 0x000B4000 - 0x00040000);
    let dfu_target: DfuTarget<256> = DfuTarget::new(dfu_partition.size(), fw_info, hw_info);
    let mut dfu = DfuDevice {
        flash: dfu_partition,
        target: dfu_target,
    };

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
            Ok(conn) => connection_events(&conn, &server, &mut dfu).await,
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
    dfu: &mut DfuDevice<'_>,
) {
    loop {
        match connection.next().await {
            GattConnectionEvent::Disconnected { reason } => {
                info!("[gatt] disconnected: {:?}", reason);
                EVENT_QUEUE.send(SystemEvent::BluetoothDisconnected).await;
                break;
            }
            GattConnectionEvent::Gatt { event } => {
                handle_gatt_event(event, connection, server, dfu).await;
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
    connection: &GattConnection<'_, '_, DefaultPacketPool>,
    server: &'_ PatinaGattServer<'_>,
    dfu: &mut DfuDevice<'_>,
) {
    match event {
        GattEvent::Read(event) => {
            event.accept().unwrap();
            info!("[gatt] read request");
        }
        GattEvent::Write(event) => {
            handle_write_event(event, connection, server, dfu).await;
        }
        _ => {
            info!("[gatt] other event");
        }
    }
}

async fn handle_write_event(
    event: WriteEvent<'_, '_, DefaultPacketPool>,
    connection: &GattConnection<'_, '_, DefaultPacketPool>,
    server: &'_ PatinaGattServer<'_>,
    dfu: &mut DfuDevice<'_>,
) {
    let handle = event.handle();

    if handle == server.cts.current_time.handle {
        let data: [u8; 10] = event.data().try_into().unwrap();
        let reply = event.accept().unwrap();
        debug!("[gatt] Write Event to Time Characteristic: {:?}", data);
        update_time(data);
        reply.send().await;
    } else if handle == server.nordic_dfu.control.handle {
        let data: Vec<u8, 256> = event.data().try_into().unwrap();
        let reply = event.accept().unwrap();
        debug!("[gatt] Write Event to DFU Control Point: {:?}", data);
        if let Ok((request, _)) = DfuRequest::decode(&data) {
            info!("[ble] request: {:?}", request);
            let (response, status) = dfu.target.process(request, &mut dfu.flash).await;
            let mut buf: [u8; 32] = [0; 32];
            if let Ok(len) = response.encode(&mut buf[..]) {
                let response = Vec::from_slice(&buf[..len]).unwrap();
                if let Err(e) = server.nordic_dfu.control.notify(&connection, &response).await {
                    info!("[gatt] Error notifying control: {:?}", e);
                }
            }
            if status == DfuStatus::DoneReset {
            info!("[gatt control] DFU Update complete, resetting device");
            Timer::after_secs(4).await;
            cortex_m::peripheral::SCB::sys_reset();
            }
        } else {
            debug!("[gatt] dfu control: unable to decode");
        }
        reply.send().await;
    } else if handle == server.nordic_dfu.packet.handle {
        let data: Vec<u8, 256> = event.data().try_into().unwrap();
        let reply = event.accept().unwrap();
        debug!("[gatt] Write Event to DFU Packet: {:?}", data);
        let request = DfuRequest::Write { data: &data[..] };
        debug!("[ble] write request: {:?}", request);
        let (response, status) = dfu.target.process(request, &mut dfu.flash).await;
        let mut buf: [u8; 32] = [0; 32];
        if let Ok(len) = response.encode(&mut buf[..]) {
            let response = Vec::from_slice(&buf[..len]).unwrap();
            if let Err(e) = server.nordic_dfu.control.notify(&connection, &response).await {
                info!("[gatt] Error notifying control: {:?}", e);
            }
            if let Err(e) = server.nordic_dfu.packet.notify(&connection, &response).await {
                info!("[gatt] Error notifying packet: {:?}", e);
            }
        }
        reply.send().await;
        if status == DfuStatus::DoneReset {
            info!("[gatt packet] DFU Update complete, resetting device");
            Timer::after_secs(4).await;
            cortex_m::peripheral::SCB::sys_reset();
        }
    } else {
        info!("[gatt] Write Event to unknown handle: {:?}, data: {:?}", handle, event.data());
    }
}