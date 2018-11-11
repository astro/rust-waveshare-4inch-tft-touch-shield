//! https://ldm-systems.ru/f/doc/catalog/HY-TFT-2,8/XPT2046.pdf

use embedded_hal::{
    digital::OutputPin,
    blocking::spi::Transfer as SpiTransfer,
    blocking::delay::DelayUs,
};

use super::super::spi::SpiDmaWrite;

mod command;
use self::command::Command;

use sh;
use core::fmt::Write;


pub mod channels {
    pub const TEMP0: u8 = 0b000;
    pub const Y: u8 = 0b001;
    pub const V_BAT: u8 = 0b010;
    pub const Z1: u8 = 0b011;
    pub const Z2: u8 = 0b100;
    pub const X: u8 = 0b101;
    pub const AUX_IN: u8 = 0b110;
    pub const TEMP1: u8 = 0b111;
}


pub struct Ts<'a, SPI: SpiDmaWrite, CS: OutputPin> {
    pub spi: SPI,
    pub cs: &'a mut CS,
}

impl<'a, SPI: SpiDmaWrite, CS: OutputPin> Ts<'a, SPI, CS> {
    /// Synchronous interface for debugging purposes
    pub fn read<D: DelayUs<u16>>(&mut self, cmd: u8, delay: &mut D) -> Result<u16, SPI::Error> {
        self.cs.set_low();
        self.spi.write_sync(&[cmd])?;
        //delay.delay_us(200);
        let mut buf = [0; 2];
        self.spi.transfer(&mut buf)?;
        self.cs.set_high();

        let r = read_12bits(&buf);
        // let r = ((buf[0] as u16) << 8) | (buf[1] as u16);
        Ok(r)
    }

    /// Synchronous interface that interleaves commands/data for
    /// higher throughput
    pub fn read_many<I>(mut self, mut iter: I) -> Result<ReadIter<'a, SPI, CS, I>, SPI::Error>
    where
        I: Iterator<Item=Command>
    {
        self.cs.set_low();

        let next_cmd = iter.next()
            .map(|command| command.into())
            .unwrap_or(0);
        self.spi.write_sync(&[next_cmd])?;

        Ok(ReadIter {
            iter,
            spi: self.spi,
            cs: self.cs,
            ended: false,
        })
    }

    pub fn read_values(self) -> Result<[u16; 4], SPI::Error> {
        fn cmd(channel: u8) -> Command {
            Command {
                channel,
                mode: false,
                ser_dfr: false,
                pd1: true,
                pd0: true,
            }
        }

        let cmds = [
            cmd(channels::X),
            cmd(channels::Y),
            cmd(channels::Z1),
            cmd(channels::Z2),
        ];
        let mut i = self.read_many(cmds.into_iter().cloned())?;
        Ok([i.next().unwrap(), i.next().unwrap(), i.next().unwrap(), i.next().unwrap()])
    }
}

pub struct ReadIter<'a, SPI: SpiDmaWrite, CS: OutputPin, I: Iterator<Item=Command>> {
    iter: I,
    spi: SPI,
    cs: &'a mut CS,
    ended: bool,
    // TODO: read_8bit: bool,
}

impl<'a, SPI: SpiDmaWrite, CS: OutputPin, I: Iterator<Item=Command>> Iterator for ReadIter<'a, SPI, CS, I> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ended {
            return None;
        }

        let next_cmd = self.iter.next()
            .map(|command| command.into())
            .unwrap_or_else(|| {
                self.ended = true;
                0
            });

        let mut buf = [0, next_cmd];
        match self.spi.transfer(&mut buf) {
            Ok(_) => {
                let r = read_12bits(&buf);
                Some(r)
            },
            Err(_) => None,
        }
    }
}

impl<'a, SPI: SpiDmaWrite, CS: OutputPin, I: Iterator<Item=Command>> Drop for ReadIter<'a, SPI, CS, I> {
    fn drop(&mut self) {
        self.cs.set_high();
    }
}

fn read_12bits(buf: &[u8; 2]) -> u16 {
    ((buf[0] as u16) << 5) | ((buf[1] as u16) >> 3)
}
