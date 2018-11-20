//! https://ldm-systems.ru/f/doc/catalog/HY-TFT-2,8/XPT2046.pdf

use embedded_hal::digital::{InputPin, OutputPin};

use super::super::spi::SpiDmaWrite;

mod command;
use self::command::Command;
mod read_commands;
use self::read_commands::{read_commands, XY_READS};


#[allow(unused)]
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

const X_PLATE_OHMS: u32 = 400;

pub struct Ts<'a, SPI: SpiDmaWrite, CS: OutputPin, Busy: InputPin> {
    pub spi: SPI,
    pub cs: &'a mut CS,
    pub busy: &'a mut Busy,
}

impl<'a, SPI: SpiDmaWrite, CS: OutputPin, Busy: InputPin> Ts<'a, SPI, CS, Busy> {
    /// Synchronous interface that interleaves commands/data for
    /// higher throughput
    pub fn read_many<I>(mut self, mut iter: I) -> Result<ReadIter<'a, SPI, CS, Busy, I>, SPI::Error>
    where
        I: Iterator<Item=Command>
    {
        self.cs.set_low();

        let next_cmd = iter.next()
            .map(|command| command.into())
            .unwrap_or(0);
        match self.spi.write_sync(&[next_cmd]) {
            Ok(_) =>
                Ok(ReadIter {
                    iter,
                    spi: self.spi,
                    cs: self.cs,
                    busy: self.busy,
                    ended: false,
                    read_mode: false,
                }),
            Err(e) => {
                self.cs.set_high();
                Err(e)
            }
        }
    }

    pub fn read_values(self) -> Result<(u16, u16, u16), SPI::Error> {
        let mut i = self.read_many(read_commands())?;

        let mut xs: [u16; XY_READS] = unsafe { core::mem::uninitialized() };
        let mut ys: [u16; XY_READS] = unsafe { core::mem::uninitialized() };

        for (x, y) in xs.iter_mut().zip(ys.iter_mut()) {
            *x = i.next().unwrap();
            *y = i.next().unwrap();
        }
        let x = nearest_avg(&xs[1..]);
        let y = nearest_avg(&ys[1..]);
        let z1 = i.next().unwrap();
        let z2 = i.next().unwrap();
        let z = if x > 0 && z1 > 0 {
            (((((z2 as u32) - (z1 as u32)) * (x as u32) * X_PLATE_OHMS) / (z1 as u32)) + 2047) >> 12
        } else {
            0
        };

        Ok((x, y, 1000u32.saturating_sub(z) as u16))
    }
}

fn nearest_avg(xs: &[u16]) -> u16 {
    if xs.len() == 0 {
        return xs[0];
    }
    fn diff(x1: u16, x2: u16) -> u16 {
        if x1 < x2 {
            x2 - x1
        } else {
            x1 - x2
        }
    }
    
    let mut x1 = xs[0];
    let mut x2 = xs[1];
    let mut d = diff(x1, x2);

    for i in 1..xs.len() {
        for j in (i + 1)..xs.len() {
            let dnew = diff(xs[i], xs[j]);
            if dnew < d {
                x1 = xs[i];
                x2 = xs[j];
                d = dnew;
            }
        }
    }

    (x1 + x2) / 2
}

pub struct ReadIter<'a, SPI: SpiDmaWrite, CS: OutputPin, Busy: InputPin, I: Iterator<Item=Command>> {
    iter: I,
    spi: SPI,
    cs: &'a mut CS,
    busy: &'a mut Busy,
    ended: bool,
    read_mode: bool,
}

impl<'a, SPI: SpiDmaWrite, CS: OutputPin, Busy: InputPin, I: Iterator<Item=Command>> Iterator for ReadIter<'a, SPI, CS, Busy, I> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ended {
            return None;
        }

        let mut next_mode = false;
        let next_cmd = self.iter.next()
            .map(|command| {
                next_mode = command.mode;
                command.into()
            })
            .unwrap_or_else(|| {
                self.ended = true;
                0
            });
        let mut buf = [0, next_cmd];
        let buf_ref = if !self.read_mode {
            &mut buf
        } else {
            &mut buf[1..]
        };

        while self.busy.is_high() {}
        match self.spi.transfer(buf_ref) {
            Ok(_) => {
                let r = if !self.read_mode {
                    read_12bits(buf_ref)
                } else {
                    buf_ref[0] as u16
                };
                self.read_mode = next_mode;
                Some(r)
            },
            Err(_) => None,
        }
    }
}

impl<'a, SPI: SpiDmaWrite, CS: OutputPin, Busy: InputPin, I: Iterator<Item=Command>> Drop for ReadIter<'a, SPI, CS, Busy, I> {
    fn drop(&mut self) {
        self.cs.set_high();
    }
}

fn read_12bits(buf: &[u8]) -> u16 {
    ((buf[1] as u16) << 5) | ((buf[0] as u16) >> 3)
}
