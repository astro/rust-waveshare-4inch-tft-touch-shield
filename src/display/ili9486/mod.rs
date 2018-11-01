//! https://www.waveshare.com/w/upload/7/78/ILI9486_Datasheet.pdf

use embedded_hal::{
    digital::OutputPin,
    blocking::spi::Transfer as SpiTransfer,
};

pub mod command;
use self::command::*;
use super::Display;

pub struct Tft<'a, SPI: SpiTransfer<u8>, DC: OutputPin, CS: OutputPin> {
    pub display: &'a mut Display<SPI, DC, CS>,
}

impl<'a, SPI: SpiTransfer<u8>, DC: OutputPin, CS: OutputPin> Tft<'a, SPI, DC, CS> {
    pub fn write(self, reg: u8) -> Result<TftWriter<'a, SPI, DC, CS>, SPI::Error> {
        let display = self.display;
        display.tft_dc.set_low();
        display.tft_cs.set_low();
        let mut buf = [0, reg];
        let result = display.spi.transfer(&mut buf);
        display.tft_dc.set_high();
        match result {
            Ok(_) => Ok(TftWriter { display }),
            Err(e) => {
                display.tft_cs.set_high();
                return Err(e);
            }
        }
    }

    pub fn write_command<C: Command>(self, c: C) -> Result<C::Response, SPI::Error> {
        let response = {
            let mut w = self.write(c.number())?;
            let mut buf = c.encode();
            w.transfer(buf.as_mut())?;
            C::decode(&buf)
        };
        Ok(response)
    }

    // fn write_reg(&mut self, reg: u8) -> Result<(), SPI::Error> {
    //     self.dc.set_low();
    //     self.cs.set_low();
    //     let mut buf = [0, reg];
    //     let result = self.display.spi.transfer(&mut buf);
    //     self.cs.set_high();
    //     result.map(|_| ())
    // }

    // fn write_data(&mut self, buf: &mut [u8]) -> Result<(), SPI::Error> {
    //     self.dc.set_high();
    //     self.cs.set_low();
    //     let result = self.display.spi.transfer(buf);
    //     self.cs.set_high();
    //     result.map(|_| ())
    // }

    // fn write_byte(&mut self, byte: u8) -> Result<(), SPI::Error> {
    //     let mut buf = [byte];
    //     self.write_data(&mut buf)
    // }

    /// TODO: use command::*
    pub fn init(&mut self) {
        self.display.tft_dc.set_low();
        self.display.tft_cs.set_high();

        // self.write_reg(0xF9);
        // self.write_byte(0x00);
        // self.write_byte(0x08);
/*
        self.write_reg(0xC0);
        self.write_byte(0x19);//VREG1OUT POSITIVE
        self.write_byte(0x1a);//VREG2OUT NEGATIVE

        self.write_reg(0xC1);
        self.write_byte(0x45);//VGH,VGL    VGH>=14V.
        self.write_byte(0x00);

        self.write_reg(0xC2);	//Normal mode, increase can change the display quality, while increasing power consumption
        self.write_byte(0x33);

        self.write_reg(0xC5);
        self.write_byte(0x00);
        self.write_byte(0x28);//VCM_REG[7:0]. <=0x80.

        self.write_reg(0xB1);//Sets the frame frequency of full color normal mode
        self.write_byte(0xA0);//0xB0 =70HZ, <=0xB0.0xA0=62HZ
        self.write_byte(0x11);

        self.write_reg(0xB4);
        self.write_byte(0x02); //2 DOT FRAME MODE,F<=70HZ.

        self.write_reg(0xB6);//
        self.write_byte(0x00);
        self.write_byte(0x42);//0 GS SS SM ISC[3:0];
        self.write_byte(0x3B);

        self.write_reg(0xB7);
        self.write_byte(0x07);

        self.write_reg(0xE0);
        self.write_byte(0x1F);
        self.write_byte(0x25);
        self.write_byte(0x22);
        self.write_byte(0x0B);
        self.write_byte(0x06);
        self.write_byte(0x0A);
        self.write_byte(0x4E);
        self.write_byte(0xC6);
        self.write_byte(0x39);
        self.write_byte(0x00);
        self.write_byte(0x00);
        self.write_byte(0x00);
        self.write_byte(0x00);
        self.write_byte(0x00);
        self.write_byte(0x00);

        self.write_reg(0xE1);
        self.write_byte(0x1F);
        self.write_byte(0x3F);
        self.write_byte(0x3F);
        self.write_byte(0x0F);
        self.write_byte(0x1F);
        self.write_byte(0x0F);
        self.write_byte(0x46);
        self.write_byte(0x49);
        self.write_byte(0x31);
        self.write_byte(0x05);
        self.write_byte(0x09);
        self.write_byte(0x03);
        self.write_byte(0x1C);
        self.write_byte(0x1A);
        self.write_byte(0x00);

        self.write_reg(0xF1);
        self.write_byte(0x36);
        self.write_byte(0x04);
        self.write_byte(0x00);
        self.write_byte(0x3C);
        self.write_byte(0x0F);
        self.write_byte(0x0F);
        self.write_byte(0xA4);
        self.write_byte(0x02);

        self.write_reg(0xF2);
        self.write_byte(0x18);
        self.write_byte(0xA3);
        self.write_byte(0x12);
        self.write_byte(0x02);
        self.write_byte(0x32);
        self.write_byte(0x12);
        self.write_byte(0xFF);
        self.write_byte(0x32);
        self.write_byte(0x00);

        self.write_reg(0xF4);
        self.write_byte(0x40);
        self.write_byte(0x00);
        self.write_byte(0x08);
        self.write_byte(0x91);
        self.write_byte(0x04);

        self.write_reg(0xF8);
        self.write_byte(0x21);
        self.write_byte(0x04);*/

/*        self.write_reg(0x3A);	//Set Interface Pixel Format
        self.write_byte(0x55);*/


        // SetGramScanWay(L2R_U2D)
        
        // Set the read / write scan direction of the frame memory
/*        self.write_reg(0xB6);
        self.write_byte(0x00);
        self.write_byte(0x22);

        self.write_reg(0x36);
        self.write_byte(0x08);*/
    }
}

pub struct TftWriter<'a, SPI: SpiTransfer<u8>, DC: OutputPin, CS: OutputPin> {
    display: &'a mut Display<SPI, DC, CS>,
}

impl<'a, SPI: SpiTransfer<u8>, DC: OutputPin, CS: OutputPin> TftWriter<'a, SPI, DC, CS> {
    pub fn transfer(&mut self, buffer: &mut [u8]) -> Result<(), SPI::Error> {
        self.display.spi.transfer(buffer)
            .map(|_| ())
    }
}

impl<'a, SPI: SpiTransfer<u8>, DC: OutputPin, CS: OutputPin> Drop for TftWriter<'a, SPI, DC, CS> {
    fn drop(&mut self) {
        self.display.tft_cs.set_high();
    }
}
