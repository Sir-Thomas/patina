use heapless::Vec;
use trouble_host::prelude::*;

#[gatt_service(uuid = "FE59")]
pub struct NordicDfuService {
    #[characteristic(uuid = "8EC90001-F315-4F60-9FB8-838830DAEA50", write, notify)]
    pub control: Vec<u8, 20>,

    #[characteristic(uuid = "8EC90002-F315-4F60-9FB8-838830DAEA50", write_without_response, notify)]
    pub packet: Vec<u8, 20>,
}