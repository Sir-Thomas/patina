//! Flash memory controller for the SPI flash chip on the PineTime.

use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_nrf::{gpio::Output, spim::Spim};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::Delay;
use spi_memory_async::series25::{Flash, FlashParameters, Identification, Status};

// TODO: This shouldn't need to be pub
pub struct XT25F32B;

impl FlashParameters for XT25F32B {
    const PAGE_SIZE: usize = 256;
    const SECTOR_SIZE: usize = 4 * 1024;
    const BLOCK_SIZE: usize = 64 * 1024;
    const CHIP_SIZE: usize = 4 * 1024 * 1024;
}

/// Flash Controller
pub struct FlashController {
    // TODO: This shouldn't need to be pub
    pub flash:
        Flash<SpiDevice<'static, NoopRawMutex, Spim<'static>, Output<'static>>, XT25F32B, Delay>,
}

impl FlashController {
    pub(crate) async fn new(
        spi: SpiDevice<'static, NoopRawMutex, Spim<'static>, Output<'static>>,
    ) -> Self {
        let flash = Flash::init(spi, Delay, 1_000, XT25F32B).await.unwrap();
        Self { flash }
    }

    /// Get the page write size of the flash chip
    pub fn page_write_size(&mut self) -> usize {
        self.flash.page_write_size()
    }

    /// Get the sector erase size of the flash chip
    pub fn sector_erase_size(&mut self) -> usize {
        self.flash.sector_erase_size()
    }

    /// Get the block erase size of the flash chip
    pub fn block_erase_size(&mut self) -> usize {
        self.flash.block_erase_size()
    }

    /// Get the total chip size of the flash chip
    pub fn chip_size(&mut self) -> usize {
        self.flash.chip_size()
    }

    /// Read device JEDEC ID
    pub async fn read_jedec_id(&mut self) -> Identification {
        self.flash.read_jedec_id().await.unwrap()
    }

    /// Read the status register of the flash chip
    pub async fn read_status(&mut self) -> Status {
        self.flash.read_status().await.unwrap()
    }

    /// Wait until the flash chip is no longer busy with a write or erase operation
    pub async fn wait_done(&mut self) {
        self.flash.wait_done().await.unwrap();
    }

    /// Read data from the flash chip into the provided buffer starting at the given offset
    pub async fn read(&mut self, offset: u32, buffer: &mut [u8]) {
        self.flash.read(offset, buffer).await.unwrap();
    }

    /// Erase a sector of the flash chip at the given offset
    pub async fn erase_sector(&mut self, offset: u32) {
        self.flash.erase_sector(offset).await.unwrap();
    }

    /// Erase a block of the flash chip at the given offset
    pub async fn erase_block(&mut self, offset: u32) {
        self.flash.erase_block(offset).await.unwrap();
    }

    /// Write data to the flash chip starting at the given offset
    pub async fn write(&mut self, offset: u32, data: &[u8]) {
        self.flash.write_bytes(offset, data).await.unwrap();
    }

    /// Erase the entire flash chip
    pub async fn erase_all(&mut self) {
        self.flash.erase_all().await.unwrap();
    }

    /// Erase a range of the flash chip from the given start offset to the given end offset
    pub async fn erase_range(&mut self, start: u32, end: u32) {
        self.flash.erase_range(start, end).await.unwrap();
    }
}
