use embedded_hal::{
    digital::OutputPin,
    blocking::{
        delay::DelayMs,
        spi::Transfer as SpiTransfer,
    },
    spi::{
        Mode as SpiMode,
        Polarity as SpiPolarity,
        Phase as SpiPhase,
    },
};

mod ili9486;
use self::ili9486::{
    command,
    Tft, TftWriter
};

/// TODO: move to ili
pub fn spi_mhz() -> u32 {
    9
}

/// TODO: move to ili
/// TODO: use MODE0 after next embedded_hal release
pub fn spi_mode() -> SpiMode {
    let polarity = SpiPolarity::IdleLow;
    let phase = SpiPhase::CaptureOnFirstTransition;
    SpiMode {
        polarity,
        phase,
    }
}

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 480;

pub struct Display<SPI: SpiTransfer<u8>, TftDc: OutputPin, TftCs: OutputPin> {
    spi: SPI,
    /// Data/Command Select Pin
    tft_dc: TftDc,
    /// Chip Select
    tft_cs: TftCs,
}

impl<SPI: SpiTransfer<u8>, TftDc: OutputPin, TftCs: OutputPin> Display<SPI, TftDc, TftCs> {
    pub fn new(spi: SPI, tft_dc: TftDc, tft_cs: TftCs) -> Result<Self, SPI::Error> {
        let mut this = Display { spi, tft_dc, tft_cs };

        this.tft().init();
        this.tft().write_command(command::SleepOut)?;
        this.tft().write_command(command::DisplayOn)?;
        this.tft().write_command(command::MemoryAccessControl {
            rgb_to_bgr: true,
            row_addr_order: false,
            col_addr_order: false,
            row_col_exchange: false,
            vert_refresh_order: false,
            horiz_refresh_order: false,
        })?;
        this.tft().write_command(command::InterfacePixelFormat {
            cpu_format: command::PixelFormat::Bpp16,
            rgb_format: command::PixelFormat::Bpp16,
        })?;

        Ok(this)
    }

    pub fn tft<'a>(&'a mut self) -> Tft<'a, SPI, TftDc, TftCs> {
        Tft { display: self }
    }

    pub fn write_pixels<'a>(&'a mut self) -> Result<TftWriter<'a, SPI, TftDc, TftCs>, SPI::Error> {
        // TODO: const from command::MemoryWrite
        self.tft().write(0x2C)
    }

    pub fn read_tft_identification(&mut self) -> Result<command::DisplayIdentification, SPI::Error> {
        self.tft().write_command(command::ReadDisplayIdentification)
    }

    pub fn set_inversion(&mut self, inverted: bool) -> Result<(), SPI::Error> {
        if inverted {
            self.tft().write_command(command::InversionOn)
        } else {
            self.tft().write_command(command::InversionOff)
        }
    }
}
