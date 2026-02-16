use heapless::Vec;
use trouble_host::prelude::*;


// #[gatt_service(uuid = "23D1BCEA-5F78-2315-DEEF-121230150000")]
#[gatt_service(uuid = "00001530-1212-EFDE-1523-785FEABCD123")]
pub struct InfinitimeDfuService {
    // Control Point
    // #[characteristic(uuid = "23D1BCEA-5F78-2315-DEEF-121231150000", write, notify)]
    #[characteristic(uuid = "00001531-1212-EFDE-1523-785FEABCD123", write, notify)]
    pub control: Vec<u8, 20>,
    // Packet
    // #[characteristic(uuid = "23D1BCEA-5F78-2315-DEEF-121232150000", write_without_response)]
    #[characteristic(uuid = "00001532-1212-EFDE-1523-785FEABCD123", write_without_response)]
    pub packet: Vec<u8, 20>,
    // Revision
    // #[characteristic(uuid = "23D1BCEA-5F78-2315-DEEF-121234150000", read, value = 8)]
    #[characteristic(uuid = "00001534-1212-EFDE-1523-785FEABCD123", read, value = 8)]
    pub revision: u16,
}