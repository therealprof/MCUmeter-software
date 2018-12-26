#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;

use heapless::consts::*;
use heapless::String;

use cortex_m_rt::entry;

use embedded_graphics::fonts::Font12x16;
use embedded_graphics::prelude::*;
use ina260::INA260;
use ssd1306::mode::GraphicsMode;
use ssd1306::Builder;

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
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        let gpioa = p.GPIOA.split();
        let gpiof = p.GPIOF.split();

        let rcc = p.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

        // Get delay provider
        let mut delay = Delay::new(cp.SYST, clocks);

        let mut led = gpioa.pa1.into_push_pull_output();
        led.set_high();
        delay.delay_ms(300_u16);
        led.set_low();
        delay.delay_ms(300_u16);
        led.set_high();

        // Disable the watchdog when the cpu is stopped under debug
        p.DBGMCU.apb1_fz.modify(|_, w| w.dbg_iwdg_stop().set_bit());

        let mut watchdog = Watchdog::new(p.IWDG);
        watchdog.start(Hertz(1));

        let scl = gpiof
            .pf1
            .into_alternate_af1()
            .internal_pull_up(true)
            .set_open_drain();
        let sda = gpiof
            .pf0
            .into_alternate_af1()
            .internal_pull_up(true)
            .set_open_drain();

        // Setup I2C1
        let i2c = I2c::i2c1(p.I2C1, (scl, sda), 400.khz());

        let i2c_bus = shared_bus::CortexMBusManager::new(i2c);

        let mut disp: GraphicsMode<_> = Builder::new().connect_i2c(i2c_bus.acquire()).into();

        disp.init().unwrap();
        disp.flush().unwrap();

        let mut ina260 = INA260::new(i2c_bus.acquire(), 0x40).unwrap();

        // Slow down sampling a bit for more accuracy
        ina260
            .set_bvconvtime_mode(ina260::BVConvTime::MS4_156)
            .unwrap();
        ina260
            .set_scconvtime_mode(ina260::SCConvTime::MS4_156)
            .unwrap();
        ina260.set_averaging_mode(ina260::Averaging::AVG16).unwrap();

        // Endless loop
        loop {
            led.set_low();

            // Read voltage
            {
                let (major, minor) = ina260.voltage_split().unwrap();
                let mut v: String<U10> = String::new();
                write!(v, "{:3}.{:05}V", major, minor).unwrap();
                disp.draw(
                    Font12x16::render_str(v.as_str())
                        .with_stroke(Some(1u8.into()))
                        .into_iter(),
                );
            }

            // Read current
            {
                let (major, minor) = ina260.current_split().unwrap();
                let mut v: String<U10> = String::new();
                write!(v, "{:3}.{:05}A", major, minor).unwrap();
                disp.draw(
                    Font12x16::render_str(v.as_str())
                        .with_stroke(Some(1u8.into()))
                        .translate(Coord::new(0, 16))
                        .into_iter(),
                );
            }

            // Read power
            {
                let (major, minor) = ina260.power_split().unwrap();
                let mut v: String<U10> = String::new();
                write!(v, "{:3}.{:05}W", major, minor).unwrap();
                disp.draw(
                    Font12x16::render_str(v.as_str())
                        .with_stroke(Some(1u8.into()))
                        .translate(Coord::new(0, 32))
                        .into_iter(),
                );
            }

            led.set_high();
            disp.flush().unwrap();

            // Reset watchdog
            watchdog.feed();
        }
    }

    loop {
        continue;
    }
}
