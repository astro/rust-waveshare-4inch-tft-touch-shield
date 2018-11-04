//! https://www.waveshare.com/w/upload/7/78/ILI9486_Datasheet.pdf

use embedded_hal::{
    digital::OutputPin,
    blocking::spi::Transfer as SpiTransfer,
};

pub mod command;
use self::command::*;

use sh;
use core::fmt::Write;

pub struct Tft<'a, SPI: SpiTransfer<u8>, DC: OutputPin, CS: OutputPin> {
    pub spi: &'a mut SPI,
    pub dc: &'a mut DC,
    pub cs: &'a mut CS,
}

impl<'a, SPI: SpiTransfer<u8>, DC: OutputPin, CS: OutputPin> Tft<'a, SPI, DC, CS> {
    pub fn write(mut self, reg: u8) -> Result<TftWriter<'a, SPI, CS>, SPI::Error> {
        let mut buf = [0, reg];

        self.dc.set_low();
        self.cs.set_low();
        let result = self.spi.transfer(&mut buf);
        self.dc.set_high();

        result.map(move |_| TftWriter {
            spi: self.spi,
            cs: self.cs,
        })
    }

    pub fn write_command<C: Command>(self, c: C) -> Result<C::Response, SPI::Error> {
        let response = {
            let mut w = self.write(C::number())?;
            let mut buf = c.encode();
            w.transfer(buf.as_mut())?;
            C::decode(&buf)
        };
        Ok(response)
    }
}


pub struct TftWriter<'a, SPI: SpiTransfer<u8>, CS: OutputPin> {
    pub spi: &'a mut SPI,
    pub cs: &'a mut CS,
}

impl<'a, SPI: SpiTransfer<u8>, CS: OutputPin> TftWriter<'a, SPI, CS> {
    pub fn transfer(&mut self, buffer: &mut [u8]) -> Result<(), SPI::Error> {
        self.spi.transfer(buffer)
            .map(|_| ())
    }
}

impl<'a, SPI: SpiTransfer<u8>, CS: OutputPin> Drop for TftWriter<'a, SPI, CS> {
    fn drop(&mut self) {
        self.cs.set_high();
    }
}
