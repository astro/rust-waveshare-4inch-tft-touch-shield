use core::mem::replace;

use embedded_hal::{
    digital::OutputPin,
    spi::{
        Mode as SpiMode,
        Polarity as SpiPolarity,
        Phase as SpiPhase,
    },
    blocking::{
        delay::DelayMs,
        spi::{
            Transfer as SpiTransfer,
            Write as SpiWrite,
        },
    },
};
use stm32f429_hal::{
    stm32f429::SPI1,
    rcc::{Clocks, APB2},
    spi::{Spi, Error, DmaWrite},
    dma::Transfer,
    time::U32Ext,
    gpio::{
        gpiof::{PF13, PF14},
        gpiod::PD14,
        gpioe::PE11,
        Output, PushPull,
    },
};

use super::spi::SpiDmaWrite;
mod ili9486;
use self::ili9486::{
    command::{self, Command},
    Tft, TftWriter
};

use sh;
use core::fmt::Write;

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
        dma::dma2,
    };

    pub type Sck = PA5<AF5>;
    pub type Miso = PA6<AF5>;
    pub type Mosi = PA7<AF5>;
    pub type ReadySpi = Spi<SPI1, (Sck, Miso, Mosi)>;
    pub type DmaStream = dma2::S3;

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
                Target::Tft => 12,
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

type TftDc = PF13<Output<PushPull>>;
type TftCs = PD14<Output<PushPull>>;
type TsCs = PF14<Output<PushPull>>;
type SdCs = PE11<Output<PushPull>>;

pub struct Display {
    spi_state: spi1::State,
    spi_dma_stream: Option<spi1::DmaStream>,
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

impl Display {
    pub fn new<D: DelayMs<u16>>(
        sck: spi1::Sck, miso: spi1::Miso, mosi: spi1::Mosi,
        spi: SPI1, spi_dma_stream: spi1::DmaStream, apb2: APB2, clocks: Clocks,
        tft_dc: TftDc, tft_cs: TftCs,
        ts_cs: TsCs, sd_cs: SdCs,
        delay: &mut D,
    ) -> Result<Self, Error> {
        let mut this = Display {
            spi_state: spi1::State::Reset(spi, sck, miso, mosi),
            spi_dma_stream: Some(spi_dma_stream),
            apb2, clocks,
            tft_dc,
            tft_cs,
            ts_cs,
            sd_cs,
        };

        this.set_all_cs_high();

        this.tft().write_command(command::SleepOut)?;
        delay.delay_ms(5);

        this.tft().write_command(command::DisplayOn)?;
        this.tft().write_command(command::MemoryAccessControl {
            rgb_to_bgr: true,
            row_addr_order: false,
            col_addr_order: false,
            row_col_exchange: false,
            vert_refresh_order: false,
            horiz_refresh_order: false,
        })?;
        let r = this.tft().write_command(command::InterfacePixelFormat {
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
    }

    pub fn tft<B: AsRef<[u8]>>(&mut self) -> Tft<DisplaySpi<B>, TftDc, TftCs> {
        self.setup_spi(spi1::Target::Tft);

        Tft {
            spi: DisplaySpi {
                spi: self.spi_state.mut_spi(),
                spi_dma_stream: &mut self.spi_dma_stream,
                dma_buffer: None,
            },
            cs: &mut self.tft_cs,
            dc: &mut self.tft_dc,
        }
    }

    pub fn write_pixels<B: AsRef<[u8]>>(&mut self) -> Result<TftWriter<DisplaySpi<B>, TftCs>, Error> {
        // TODO: send empty memorywrite
        self.tft::<B>().writer(command::MemoryWrite::number())
    }

    pub fn set_inversion(&mut self, inverted: bool) -> Result<(), Error> {
        if inverted {
            self.tft().write_command(command::InversionOn)
        } else {
            self.tft().write_command(command::InversionOff)
        }
    }
}

pub struct DisplaySpi<'a, B: AsRef<[u8]>> {
    spi: &'a mut spi1::ReadySpi,
    spi_dma_stream: &'a mut Option<spi1::DmaStream>,
    dma_buffer: Option<B>,
}

impl<'a, Buf: AsRef<[u8]>> SpiDmaWrite for DisplaySpi<'a, Buf> {
    type Error = Error;
    type DmaBuffer = Buf;

    fn write_sync<B: AsRef<[u8]>>(&mut self, buffer: B) -> Result<(), Self::Error> {
        self.spi.write(buffer.as_ref())
    }

    fn write_async(&mut self, buffer: Buf) -> Result<(), Self::Error> {
        // Clear previous
        self.flush()?;

        if buffer.as_ref().len() == 0 {
            return Ok(());
        }

        let stream = self.spi_dma_stream.take().unwrap();
        let stream =
            self.spi.dma_write::<_, _, SPI1, spi1::DmaStream, _, _>(stream, buffer)
            .wait()
            .unwrap_or_else(|stream| {
                let mut hstdout = sh::hio::hstdout().unwrap();
                writeln!(hstdout, "dma err").unwrap();
                stream
            });
        for _ in 0..100 {
            cortex_m::asm::nop();
        }
        *self.spi_dma_stream = Some(stream);

        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
