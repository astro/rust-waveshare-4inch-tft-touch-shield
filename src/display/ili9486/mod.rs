//! https://www.waveshare.com/w/upload/7/78/ILI9486_Datasheet.pdf

use embedded_hal::digital::OutputPin;

pub mod command;
use self::command::*;
use super::super::spi::SpiDmaWrite;


pub struct Tft<'a, SPI: SpiDmaWrite, DC: OutputPin, CS: OutputPin> {
    pub spi: SPI,
    pub dc: &'a mut DC,
    pub cs: &'a mut CS,
}

impl<'a, SPI: SpiDmaWrite, DC: OutputPin, CS: OutputPin> Tft<'a, SPI, DC, CS> {
    pub fn writer(mut self, reg: u8) -> Result<TftWriter<'a, SPI, CS>, SPI::Error> {
        let buf = [0, reg];

        self.dc.set_low();
        self.cs.set_low();
        let result = self.spi.write_sync(buf);
        self.dc.set_high();

        result.map(move |_| TftWriter {
            spi: self.spi,
            cs: self.cs,
        })
    }
}

impl<'a, B: AsRef<[u8]>, SPI: SpiDmaWrite<DmaBuffer=B>, DC: OutputPin, CS: OutputPin> Tft<'a, SPI, DC, CS> {
    pub fn write_command<C: Command<Buffer=B>>(self, c: C) -> Result<(), SPI::Error> {
        let mut w = self.writer(C::number())?;
        let buf = c.encode();
        w.write(buf)
    }
}


pub struct TftWriter<'a, SPI: SpiDmaWrite, CS: OutputPin> {
    pub spi: SPI,
    pub cs: &'a mut CS,
}

impl<'a, SPI: SpiDmaWrite, CS: OutputPin> TftWriter<'a, SPI, CS> {
    pub fn write(&mut self, buffer: SPI::DmaBuffer) -> Result<(), SPI::Error> {
        self.spi.write_async(buffer)
    }
}

impl<'a, SPI: SpiDmaWrite, CS: OutputPin> Drop for TftWriter<'a, SPI, CS> {
    fn drop(&mut self) {
        self.spi.flush()
            .unwrap_or_else(|_| ());
        cortex_m::asm::delay(64);
        self.cs.set_high();
    }
}
