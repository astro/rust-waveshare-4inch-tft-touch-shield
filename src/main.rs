#![no_std]
#![no_main]

extern crate panic_semihosting;
extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate stm32f429_hal;
extern crate cortex_m_semihosting as sh;
extern crate embedded_hal;
extern crate nb;
extern crate vga_framebuffer;

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
    blocking::delay::DelayUs,
};

mod spi;
mod display;
use display::{Display, WIDTH, HEIGHT, console::Console, ScanLine};


#[entry]
fn main() -> ! {
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
    let ts_pen = gpioe.pe13.into_floating_input(&mut gpioe.moder, &mut gpioe.pupdr);
    let ts_busy = gpioe.pe9.into_floating_input(&mut gpioe.moder, &mut gpioe.pupdr);
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
        ts_pen, ts_busy, ts_cs,
        sd_cs,
        &mut delay
    ).expect("display");
    let mut cons = Console::new();

    let mut t = 0;
    let mut touch = None;
    let mut prev_touch = touch.clone();
    let x_min = 460;
    let y_min = 800;
    let x_max = 4000;
    let y_max = 4000;
    let mut z_repeat = 0;
    loop {
        led_red.set_high();
        let is_input = display.ts_input();
        if is_input {
            let (x, y, z) = display.ts().read_values().unwrap();
            if z > 0 {
                z_repeat += 1;
            } else {
                z_repeat = 0;
            }
            if z_repeat > 5 {
                writeln!(&mut cons, "x: {} y: {}", x, y).unwrap();
                let x = (x as usize).max(x_min).min(x_max);
                let y = (y as usize).max(y_min).min(y_max);
                touch = Some((
                    WIDTH * (x - x_min) / (x_max - x_min),
                    HEIGHT * ((y_max - y_min) - (y - y_min)) / (y_max - y_min),
                    z
                ));
            } else {
                touch = None;
            }
        } else {
            writeln!(&mut cons, "no input").unwrap();
        }
        led_red.set_low();

        if touch != prev_touch || t < 2 {
            prev_touch = touch.clone();
            writeln!(&mut cons, "touch: {:?}", touch).unwrap();

            led_red.set_high();
            // display.set_pixel_area(50, 100, 50, 100).unwrap();
            let mut w = display.write_pixels()
                .expect("write_pixels");
            led_red.set_low();

            for y in 0..HEIGHT {
                led_blue.set_high();
                let scanline = ScanLine::new(|x| {
                    let tint = 255u8.saturating_sub((y >> 1).min(255) as u8);
                    let mut r = tint >> 2;
                    let mut g = 0;
                    let mut b = tint >> 1;
                    match touch {
                        Some((px, py, _))
                            if (x == px) || (y == py) => {
                                r = 0;
                                g = 255;
                                b = 0;
                            }
                        _ => {}
                    }

                    if cons.get_pixel(x, y) {
                        r = 255;
                        g = 255;
                        b = 255;
                    }

                    (r, g, b)
                });
                led_blue.set_low();

                led_green.set_high();
                w.write(scanline)
                    .expect("write");
                led_green.set_low();
            }
        }

        t += 1;
    }
}
