use core::mem::replace;

use embedded_hal::{
    digital::OutputPin,
    // blocking::delay::DelayMs,
    spi::{
        Mode as SpiMode,
        Polarity as SpiPolarity,
        Phase as SpiPhase,
    },
};
use stm32f429_hal::{
    stm32f429::SPI1,
    rcc::{Clocks, APB2},
    spi::{Spi, Error},
    time::U32Ext,
};

mod ili9486;
use self::ili9486::{
    command::{self, Command},
    Tft, TftWriter
};

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 480;


/// TODO: use MODE0 after next embedded_hal release
fn spi_mode0() -> SpiMode {
    let polarity = SpiPolarity::IdleLow;
    let phase = SpiPhase::CaptureOnFirstTransition;
    SpiMode {
        polarity,
        phase,
    }
}

mod spi1 {
    use embedded_hal::spi::Mode as SpiMode;
    use stm32f429_hal::{
        stm32f429::SPI1,
        spi::Spi,
        gpio::{
            AF5,
            gpioa::{PA5, PA6, PA7},
        },
    };

    pub type Sck = PA5<AF5>;
    pub type Miso = PA6<AF5>;
    pub type Mosi = PA7<AF5>;
    pub type ReadySpi = Spi<SPI1, (Sck, Miso, Mosi)>;

    pub enum State {
        Reset(SPI1, Sck, Miso, Mosi),
        Ready(Target, ReadySpi),
        Invalid,
    }

    impl State {
        pub fn mut_spi(&mut self) -> &mut ReadySpi {
            match self {
                State::Ready(_, spi) => spi,
                State::Reset(_, _, _, _) => panic!("SPI not setup"),
                State::Invalid => unreachable!(),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Target {
        /// TFT controller
        Tft,
        /// Touch screen
        Ts,
        /// SD card slot
        Sd,
    }

    impl Target {
        pub fn mhz(&self) -> u32 {
            match *self {
                Target::Tft => 9,  // TODO: 16?
                Target::Ts => 2,
                Target::Sd => 8,
                _ => panic!("TODO: Not implemented"),
            }
        }

        pub fn mode(&self) -> SpiMode {
            super::spi_mode0()
        }
    }
}

pub struct Display<TftDc: OutputPin, TftCs: OutputPin, TsCs: OutputPin, SdCs: OutputPin> {
    spi_state: spi1::State,
    apb2: APB2,
    clocks: Clocks,
    /// Data/Command Select Pin
    tft_dc: TftDc,
    /// Chip Select
    tft_cs: TftCs,
    /// Chip Select
    ts_cs: TsCs,
    /// Chip Select
    sd_cs: SdCs,
}

impl<TftDc: OutputPin, TftCs: OutputPin, TsCs: OutputPin, SdCs: OutputPin> Display<TftDc, TftCs, TsCs, SdCs> {
    pub fn new(
        sck: spi1::Sck, miso: spi1::Miso, mosi: spi1::Mosi,
        spi: SPI1, apb2: APB2, clocks: Clocks,
        tft_dc: TftDc, tft_cs: TftCs,
        ts_cs: TsCs, sd_cs: SdCs
    ) -> Result<Self, Error> {
        let mut this = Display {
            spi_state: spi1::State::Reset(spi, sck, miso, mosi),
            apb2, clocks,
            tft_dc,
            tft_cs,
            ts_cs,
            sd_cs,
        };

        this.set_all_cs_high();
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

    fn set_all_cs_high(&mut self) {
        self.tft_cs.set_high();
        self.ts_cs.set_high();
        self.sd_cs.set_high();
    }

    /// Lazy switching of SPI modes
    fn setup_spi(&mut self, target: spi1::Target) {
        let spi_state = replace(&mut self.spi_state, spi1::State::Invalid);
        let (spi1, (sck, miso, mosi)) =
             match spi_state {
                 spi1::State::Ready(current_target, spi) =>
                     if current_target == target {
                         // All is well
                         self.spi_state = spi1::State::Ready(current_target, spi);
                         return;
                     } else {
                         self.set_all_cs_high();
                         // TODO: flush DMA?
                         spi.free()
                     },
                 spi1::State::Reset(spi1, sck, miso, mosi) =>
                     (spi1, (sck, miso, mosi)),
                 spi1::State::Invalid =>
                     unreachable!(),
             };

        let spi = Spi::spi1(
            spi1, (sck, miso, mosi),
            target.mode(), target.mhz().mhz(),
            self.clocks, &mut self.apb2
        );
        self.spi_state = spi1::State::Ready(target, spi);

        match target {
            spi1::Target::Tft =>
                self.tft_cs.set_low(),
            spi1::Target::Ts =>
                self.ts_cs.set_low(),
            spi1::Target::Sd =>
                self.sd_cs.set_low(),
        }
    }

    pub fn tft<'a>(&'a mut self) -> Tft<'a, spi1::ReadySpi, TftDc> {
        self.setup_spi(spi1::Target::Tft);

        let spi = self.spi_state.mut_spi();
        Tft {
            dc: &mut self.tft_dc,
            spi: spi,
        }
    }

    pub fn write_pixels<'a>(&'a mut self) -> Result<TftWriter<'a, spi1::ReadySpi>, Error> {
        self.tft().write(command::MemoryWrite::number())
    }

    pub fn read_tft_identification(&mut self) -> Result<command::DisplayIdentification, Error> {
        self.tft().write_command(command::ReadDisplayIdentification)
    }

    pub fn set_inversion(&mut self, inverted: bool) -> Result<(), Error> {
        if inverted {
            self.tft().write_command(command::InversionOn)
        } else {
            self.tft().write_command(command::InversionOff)
        }
    }
}
