//! https://www.waveshare.com/w/upload/7/78/ILI9486_Datasheet.pdf

use embedded_hal::{
    digital::OutputPin,
    blocking::spi::Transfer as SpiTransfer,
};

pub mod command;
use self::command::*;
use super::Display;
use super::super::spi::SpiDmaWrite;

use sh;
use core::fmt::Write;

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

    pub fn write_command<C: Command>(self, c: C) -> Result<C::Response, SPI::Error> {
        let response = {
            let mut w = self.writer(C::number())?;
            let mut buf = c.encode();
            w.write(buf.as_mut())?;
            C::decode(&buf)
        };
        Ok(response)
    }
}


pub struct TftWriter<'a, SPI: SpiDmaWrite, CS: OutputPin> {
    pub spi: SPI,
    pub cs: &'a mut CS,
}

impl<'a, SPI: SpiDmaWrite, CS: OutputPin> TftWriter<'a, SPI, CS> {
    pub fn write<B: AsRef<[u8]>>(&mut self, buffer: B) -> Result<(), SPI::Error> {
        self.spi.write_async(buffer)
    }
}

impl<'a, SPI: SpiDmaWrite, CS: OutputPin> Drop for TftWriter<'a, SPI, CS> {
    fn drop(&mut self) {
        self.spi.flush();
        self.cs.set_high();
    }
}
