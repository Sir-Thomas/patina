use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, with_timeout};
use nrf_sdc::{SoftdeviceController, mpsl::MultiprotocolServiceLayer};
use pinetime_bsp::ble::BleController;
use static_cell::StaticCell;
use trouble_host::{HostResources, prelude::*};

use crate::signals::ADJUST_TIME;

// We have to pretend to be InfiniTime to get companion apps to connect
// TODO: Find an app that can connect to a custom device name, or implement our own companion app
const NAME: &str = "InfiniTime";

const L2CAP_MTU: usize = 27;
const L2CAP_CHANNELS_MAX: usize = 2;
type BleResources = HostResources<DefaultPacketPool, L2CAP_CHANNELS_MAX, L2CAP_MTU>;
static RESOURCES: StaticCell<BleResources> = StaticCell::new();
static STACK: StaticCell<Stack<'static, SoftdeviceController, DefaultPacketPool>> = StaticCell::new();


pub fn ble_runner(bluetooth: BleController, spawner: Spawner) {

    let address: Address = Address::random([0xff, 0x8f, 0x2a, 0x05, 0xe4, 0xff]);

    let resources = RESOURCES.init(BleResources::new());
    let stack = STACK.init(trouble_host::new(bluetooth.sdc, resources).set_random_address(address));

    let Host { peripheral, runner, .. } = stack.build();

    spawner.must_spawn(mpsl_task(bluetooth.mpsl));
    spawner.must_spawn(ble_task(runner));
    spawner.must_spawn(advertise_task(stack, peripheral));
}

#[embassy_executor::task]
async fn mpsl_task(mpsl: &'static MultiprotocolServiceLayer<'static>) -> ! {
    mpsl.run().await
}

#[embassy_executor::task]
async fn ble_task(mut runner: Runner<'static, SoftdeviceController<'static>, DefaultPacketPool>) {
    runner.run().await.unwrap();
}

#[embassy_executor::task]
async fn advertise_task(
    stack: &'static Stack<'static, SoftdeviceController<'static>, DefaultPacketPool>,
    mut peripheral: Peripheral<'static, SoftdeviceController<'static>, DefaultPacketPool>,
) {
    let mut advertiser_data = [0; 31];
    AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::CompleteLocalName(NAME.as_bytes()),
        ],
        &mut advertiser_data[..],
    ).unwrap();
    loop {
        info!("[ble] advertising");
        let advertiser = peripheral.advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &advertiser_data[..],
                scan_data: &[],
            },
        ).await.unwrap();
        match advertiser.accept().await {
            Ok(connection) => process_connection(stack, connection).await,
            Err(e) => {
                info!("Error advertising: {:?}", e);
            }
        }
    }
}

async fn process_connection(
    stack: &'static Stack<'static, SoftdeviceController<'static>, DefaultPacketPool>,
    connection: Connection<'static, DefaultPacketPool>,
) {
    sync_time(stack, connection.clone()).await;

    loop {
        let event = connection.next().await;
        match event {
            ConnectionEvent::Disconnected { reason } => {
                defmt::info!("[ble] disconnected: {:?}", reason);
                break;
            }
            _ => {}
        }
    }
}

async fn sync_time(
    stack: &'static Stack<'static, SoftdeviceController<'static>, DefaultPacketPool>,
    conn: Connection<'static, DefaultPacketPool>
) {
    info!("[ble] synchronizing time");
    let client = GattClient::<_, DefaultPacketPool, 10>::new(stack, &conn).await.unwrap();
    match select(
        client.task(),
        with_timeout(Duration::from_secs(8), async {
            let services = client.services_by_uuid(&Uuid::new_short(0x1805)).await?;
            for service in &services {
                info!("[ble] found service: {:?}", service);
            }
            if let Some(service) = services.first() {
                info!("[ble] found current time service");
                let c: Characteristic<u8> = client
                    .characteristic_by_uuid(&service, &Uuid::new_short(0x2a2b))
                    .await?;

                let mut data = [0; 10];
                client.read_characteristic(&c, &mut data[..]).await?;

                if let Some(time) = parse_time(data) {
                    let (h, m, s) = time.as_hms();
                    info!("[ble] received time: {:02}:{:02}:{:02}", h, m, s);
                    ADJUST_TIME.signal(time);
                }
            } else {
                info!("[ble] current time service not found");
            }
            Ok::<(), BleHostError<nrf_sdc::Error>>(())
        }),
    )
    .await
    {
        Either::First(_) => panic!("[ble] gatt client exited prematurely"),
        Either::Second(Ok(_)) => {
            info!("[ble] time sync completed");
        }
        Either::Second(Err(e)) => {
            info!("[ble] time sync error: {:?}", e);
        }
    }
}

fn parse_time(data: [u8; 10]) -> Option<time::PrimitiveDateTime> {
    let year = u16::from_le_bytes([data[0], data[1]]);
    let month = data[2];
    let day = data[3];
    let hour = data[4];
    let minute = data[5];
    let second = data[6];
    let _weekday = data[7];
    let secs_frac = data[8];

    if let Ok(month) = month.try_into() {
        let date = time::Date::from_calendar_date(year as i32, month, day);
        let micros = secs_frac as u32 * 1000000 / 256;
        let time = time::Time::from_hms_micro(hour, minute, second, micros);
        if let (Ok(time), Ok(date)) = (time, date) {
            let dt = time::PrimitiveDateTime::new(date, time);
            return Some(dt);
        }
    }
    info!("[ble] failed to parse time data: {:?}", data);
    None
}