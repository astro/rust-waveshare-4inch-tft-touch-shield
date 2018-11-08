#![no_std]
#![no_main]

extern crate panic_semihosting;
extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate stm32f429_hal;
extern crate cortex_m_semihosting as sh;
extern crate embedded_hal;

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
use display::{Display, WIDTH, HEIGHT};

#[entry]
fn main() -> ! {
    let mut hstdout = sh::hio::hstdout().unwrap();

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
    // delay.delay_us(9u16);
    delay.delay_ms(50u16);
    lcd_rst.set_high();
    // delay.delay_us(300u16);
    delay.delay_ms(50u16);

    lcd_bl.set_high();
    let mut display = Display::new(
        sck, miso, mosi,
        dp.SPI1, dma_streams.s3, rcc.apb2, clocks,
        lcd_dc, lcd_cs,
        ts_cs, sd_cs,
        &mut delay
    ).expect("display");
    // writeln!(hstdout, "Ident: {:?}", display.read_tft_identification().unwrap()).unwrap();

    let mut t = 0;
    loop {
        // writeln!(hstdout, "Loop: {}", t).unwrap();
        led_red.set_low();
        let mut w = display.write_pixels()
            .expect("write_pixels");
        for y in 0..HEIGHT {
            let mut buf = [0u8; 2 * WIDTH];
            led_blue.set_high();

            for x in 0..WIDTH {
                // let r = 255 * (((x + t) / 32) % 2) as u8;
                // let g = 255 * (((y + t) / 32) % 2) as u8;
                // let b = 255 * (((x - t) / 32) % 2 + ((y - t) / 32) % 2) as u8;
                let r = (x as u8).wrapping_add(t as u8);
                let g = (x as u8).wrapping_sub(y as u8).wrapping_add(t as u8);
                let b = (y as u8).wrapping_sub(t as u8);
                let i = x * 2;
                buf[i] = (r & 0xF8) | (g >> 5);
                buf[i + 1] = ((g & 0x1C) << 3) | (b >> 5);
                // buf[i] = r << 2;
                // buf[i + 1] = g << 2;
                // buf[i + 2] = b << 2;
            }
            led_blue.set_low();
            // writeln!(hstdout, "{}: {:?}", y, &buf[..]);
            led_green.set_high();
            w.write(&buf[..])
                .expect("write");
            led_green.set_low();
        }
        // drop(w);
        led_red.set_high();

        t += 1;
    }
}
