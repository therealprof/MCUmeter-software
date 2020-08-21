#![no_main]
#![no_std]

use panic_halt as _;

use stm32f0xx_hal as hal;

use heapless::consts::*;
use heapless::String;

use cortex_m_rt::entry;

use embedded_graphics::{
    fonts::{Font12x16, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    style::TextStyleBuilder,
};
use ina260::INA260;
use ssd1306::mode::GraphicsMode;
use ssd1306::{Builder, I2CDIBuilder};

use crate::hal::delay::Delay;
use crate::hal::i2c::*;
use crate::hal::prelude::*;
use crate::hal::stm32;
use crate::hal::time::Hertz;
use crate::hal::watchdog::Watchdog;

use cortex_m::peripheral::Peripherals;

use core::fmt::Write;

#[entry]
fn main() -> ! {
    if let (Some(mut p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        cortex_m::interrupt::free(|cs| {
            let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut p.FLASH);

            let gpioa = p.GPIOA.split(&mut rcc);
            let gpiof = p.GPIOF.split(&mut rcc);

            // Get delay provider
            let mut delay = Delay::new(cp.SYST, &rcc);

            let mut led = gpioa.pa1.into_push_pull_output(cs);
            led.set_high().ok();
            delay.delay_ms(300_u16);
            led.set_low().ok();
            delay.delay_ms(300_u16);
            led.set_high().ok();

            // Disable the watchdog when the cpu is stopped under debug
            p.DBGMCU.apb1_fz.modify(|_, w| w.dbg_iwdg_stop().set_bit());

            let mut watchdog = Watchdog::new(p.IWDG);
            watchdog.start(Hertz(1));

            let scl = gpiof
                .pf1
                .into_alternate_af1(cs)
                .internal_pull_up(cs, true)
                .set_open_drain(cs);
            let sda = gpiof
                .pf0
                .into_alternate_af1(cs)
                .internal_pull_up(cs, true)
                .set_open_drain(cs);

            // Setup I2C1
            let i2c = I2c::i2c1(p.I2C1, (scl, sda), 400.khz(), &mut rcc);

            let i2c_bus = shared_bus::CortexMBusManager::new(i2c);

            let text_style = TextStyleBuilder::new(Font12x16)
                .text_color(BinaryColor::On)
                .build();

            let interface = I2CDIBuilder::new().init(i2c_bus.acquire());
            let mut disp: GraphicsMode<_> = Builder::new().connect(interface).into();

            disp.init().map_err(drop).unwrap();
            disp.flush().map_err(drop).unwrap();

            let mut ina260 = INA260::new(i2c_bus.acquire(), 0x40).map_err(drop).unwrap();

            // Slow down sampling a bit for more accuracy
            ina260
                .set_bvconvtime_mode(ina260::BVConvTime::MS4_156)
                .unwrap();
            ina260
                .set_scconvtime_mode(ina260::SCConvTime::MS4_156)
                .unwrap();
            ina260
                .set_averaging_mode(ina260::Averaging::AVG16)
                .map_err(drop)
                .unwrap();

            // Endless loop
            loop {
                led.set_low().ok();

                // Clear screen contents
                disp.clear();

                // Read voltage
                {
                    let (major, minor) = ina260.voltage_split().map_err(drop).unwrap();
                    let mut v: String<U10> = String::new();
                    write!(v, "{:3}.{:05}V", major, minor)
                        .map_err(drop)
                        .ok();

                    Text::new(v.as_str(), Point::new(0, 0))
                        .into_styled(text_style)
                        .draw(&mut disp)
                        .ok();
                }

                // Read current
                {
                    let (major, minor) = ina260.current_split().map_err(drop).unwrap();
                    let mut v: String<U10> = String::new();
                    write!(v, "{:3}.{:05}A", major, minor)
                        .map_err(drop)
                        .ok();

                    Text::new(v.as_str(), Point::new(0, 16))
                        .into_styled(text_style)
                        .draw(&mut disp)
                        .ok();
                }

                // Read power
                {
                    let (major, minor) = ina260.power_split().map_err(drop).unwrap();
                    let mut v: String<U10> = String::new();
                    write!(v, "{:3}.{:05}W", major, minor)
                        .map_err(drop)
                        .ok();

                    Text::new(v.as_str(), Point::new(0, 32))
                        .into_styled(text_style)
                        .draw(&mut disp)
                        .ok();
                }

                disp.flush().map_err(drop).unwrap();

                // Reset watchdog
                watchdog.feed();
            }
        });
    }

    loop {
        continue;
    }
}
