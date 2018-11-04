//! https://www.waveshare.com/w/upload/7/78/ILI9486_Datasheet.pdf

use embedded_hal::{
    digital::OutputPin,
    blocking::spi::Transfer as SpiTransfer,
};

pub mod command;
use self::command::*;

pub struct Tft<'a, SPI: SpiTransfer<u8>, DC: OutputPin> {
    pub dc: &'a mut DC,
    pub spi: &'a mut SPI,
}

impl<'a, SPI: SpiTransfer<u8>, DC: OutputPin> Tft<'a, SPI, DC> {
    pub fn write(mut self, reg: u8) -> Result<TftWriter<'a, SPI>, SPI::Error> {
        self.dc.set_low();
        let mut buf = [0, reg];
        let result = self.spi.transfer(&mut buf);
        self.dc.set_high();
        result.map(move |_| TftWriter {
            spi: self.spi,
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

pub struct TftWriter<'a, SPI: SpiTransfer<u8>> {
    pub spi: &'a mut SPI,
}

impl<'a, SPI: SpiTransfer<u8>> TftWriter<'a, SPI> {
    pub fn transfer(&mut self, buffer: &mut [u8]) -> Result<(), SPI::Error> {
        self.spi.transfer(buffer)
            .map(|_| ())
    }
}
