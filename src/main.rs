#![no_std]
#![no_main]

extern crate panic_semihosting;
extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate stm32f429_hal;
extern crate cortex_m_semihosting as sh;
extern crate embedded_hal;
#[macro_use]
extern crate nb;

use core::fmt::Write;
use stm32f429_hal::{
    stm32f429,
    rcc::RccExt,
    flash::FlashExt,
    gpio::GpioExt,
    delay::Delay,
    time::U32Ext,
    dma::DmaExt,
};
use embedded_hal::{
    digital::OutputPin,
    blocking::delay::{DelayUs, DelayMs},
};

mod spi;
mod display;
use display::{Display, WIDTH, HEIGHT, ili9486::command};

struct ScanLine([u8; 2 * WIDTH]);
impl AsRef<[u8]> for ScanLine {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

const COLORS: &'static [[u8; 3]] = &[
    [255, 0, 0],
    [255, 255, 0],
    [0, 255, 0],
    [0, 255, 255],
];

#[entry]
fn main() -> ! {
    // let mut hstdout = sh::hio::hstdout().unwrap();

    let mut cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f429::Peripherals::take().unwrap();

    
    cp.SCB.enable_icache();
    cp.SCB.enable_dcache(&mut cp.CPUID);

    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let clocks = rcc.cfgr
        .sysclk(72.mhz())
        .pclk1(36.mhz())
        .pclk2(72.mhz())
        .freeze(&mut flash.acr);
    // writeln!(hstdout, "Clocks: {:?}", clocks).unwrap();
    
    let mut delay = Delay::new(cp.SYST, clocks);

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb1);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb1);
    let mut gpiod = dp.GPIOD.split(&mut rcc.ahb1);
    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb1);
    let mut gpiof = dp.GPIOF.split(&mut rcc.ahb1);

    let mut led_green = gpiob.pb0.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let mut led_blue = gpiob.pb7.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    let mut led_red = gpiob.pb14.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    let mut lcd_bl = gpiod.pd15.into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper);
    let mut lcd_rst = gpiof.pf12.into_push_pull_output(&mut gpiof.moder, &mut gpiof.otyper);
    let lcd_dc = gpiof.pf13.into_push_pull_output(&mut gpiof.moder, &mut gpiof.otyper);
    let lcd_cs = gpiod.pd14.into_push_pull_output(&mut gpiod.moder, &mut gpiod.otyper);
    let ts_cs = gpiof.pf14.into_push_pull_output(&mut gpiof.moder, &mut gpiof.otyper);
    let sd_cs = gpioe.pe11.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

    let mosi = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let miso = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let sck = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);

    let dma_streams = dp.DMA2.split(&mut rcc.ahb1);

    lcd_rst.set_low();
    delay.delay_us(9u16);
    lcd_rst.set_high();
    delay.delay_us(300u16);

    lcd_bl.set_high();
    let mut display = Display::new(
        sck, miso, mosi,
        dp.SPI1, dma_streams.s3, rcc.apb2, clocks,
        lcd_dc, lcd_cs,
        ts_cs, sd_cs,
        &mut delay
    ).expect("display");

    let mut t = 0;
    let mut ht = 0;
    let mut hist = [[0u16; 4]; HEIGHT];
    loop {
        led_red.set_high();
        for _ in 0..8 {
            let vs = display.ts().read_values().unwrap();
            hist[ht % HEIGHT] = [
                // vs[0] * (WIDTH as u16) / 4095,
                // vs[1] * (WIDTH as u16) / 4095,
                // vs[2] * (WIDTH as u16) / 4095,
                // vs[3] * (WIDTH as u16) / 4095,
                vs[0] >> 4,
                vs[1] >> 4,
                vs[2] >> 4,
                vs[3] >> 4,
            ];
            ht += 1;
            if ht >= HEIGHT {
                ht = 0;
            }
        }
        led_red.set_low();

        {
            led_green.set_high();
            let mut w = display.write_pixels::<ScanLine>()
                .expect("write_pixels");
            led_green.set_low();

            for y in 0..HEIGHT {
                led_blue.set_high();
                let mut buf: [u8; 2 * WIDTH] = unsafe { core::mem::uninitialized() };
                let mut hty = ht as isize - 1 - y as isize;
                if hty < 0 { hty += HEIGHT as isize }
                let h = &hist[hty as usize];
                let grid = y % (HEIGHT / 8) == 0;

                let mut i = 0;
                for x in 0..WIDTH {
                    let mut r = 0;
                    let mut g = 0;
                    let mut b = 0;
                    if grid || x % (WIDTH / 8) == 0 {
                        b = 128;
                    }

                    for (hi, hx) in h.iter().enumerate() {
                        let hx = *hx as usize;
                        if x == hx {
                            r = COLORS[hi][0];
                            g = COLORS[hi][1];
                            b = COLORS[hi][2];
                        }
                    }
                    
                    buf[i] = (r & 0xF8) | (g >> 5);
                    buf[i + 1] = ((g & 0x1C) << 3) | (b >> 5);
                    i += 2;
                }
                led_blue.set_low();

                led_green.set_high();
                w.write(ScanLine(buf))
                    .expect("write");
                led_green.set_low();
            }
        }

        // let x = display.ts().read(0xD3, &mut delay).unwrap();
        // let y = display.ts().read(0x93, &mut delay).unwrap();
        // writeln!(hstdout, "{}\t{}", x, y);
        // writeln!(hstdout, "{:?}", display.ts().read_values().unwrap()).unwrap();

        t += 1;
    }
}
