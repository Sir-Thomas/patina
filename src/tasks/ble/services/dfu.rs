use heapless::Vec;
use trouble_host::prelude::*;


#[gatt_service(uuid = "23D1BCEA-5F78-2315-DEEF-121230150000")]
pub struct InfinitimeDfuService {
    // Control Point
    #[characteristic(uuid = "23D1BCEA-5F78-2315-DEEF-121231150000", write, notify)]
    control: Vec<u8, 20>,

    // Packet
    #[characteristic(uuid = "23D1BCEA-5F78-2315-DEEF-121232150000", write_without_response)]
    packet: Vec<u8, 20>,

    // Revision
    #[characteristic(uuid = "23D1BCEA-5F78-2315-DEEF-121234150000", read, value = 8)]
    revision: u16,
}