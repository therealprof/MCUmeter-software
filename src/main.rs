#![no_main]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate panic_halt;

extern crate stm32f042_hal as hal;

extern crate shared_bus;

extern crate embedded_graphics;
extern crate heapless;
extern crate ina260;
extern crate ssd1306;

use heapless::consts::*;
use heapless::String;

use cortex_m_rt::entry;

use embedded_graphics::fonts::Font12x16;
use embedded_graphics::prelude::*;
use ina260::INA260;
use ssd1306::mode::GraphicsMode;
use ssd1306::Builder;

use hal::delay::Delay;
use hal::i2c::*;
use hal::prelude::*;
use hal::stm32;

use cortex_m::peripheral::Peripherals;

use core::fmt::Write;

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        let gpioa = p.GPIOA.split();
        let gpiof = p.GPIOF.split();

        let mut rcc = p.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

        // Get delay provider
        let mut delay = Delay::new(cp.SYST, clocks);

        let mut led = gpioa.pa1.into_push_pull_output();
        led.set_high();
        delay.delay_ms(300_u16);
        led.set_low();
        delay.delay_ms(300_u16);
        led.set_high();

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
        let mut i2c = I2c::i2c1(p.I2C1, (scl, sda), 400.khz());

        let i2c_bus = shared_bus::CortexMBusManager::new(i2c);

        let mut disp: GraphicsMode<_> = Builder::new().connect_i2c(i2c_bus.acquire()).into();

        disp.init().unwrap();
        disp.flush().unwrap();

        let mut ina260 = INA260::new(i2c_bus.acquire(), 0x40).unwrap();

        // Endless loop
        loop {
            led.set_low();

            // Read voltage
            {
                let (major, minor) = ina260.voltage_split().unwrap();
                let mut v: String<U10> = String::new();
                let _ = write!(v, "{:3}.{:05}V", major, minor);
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
                let _ = write!(v, "{:3}.{:05}A", major, minor);
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
                let _ = write!(v, "{:3}.{:05}W", major, minor);
                disp.draw(
                    Font12x16::render_str(v.as_str())
                        .with_stroke(Some(1u8.into()))
                        .translate(Coord::new(0, 32))
                        .into_iter(),
                );
            }

            led.set_high();
            disp.flush().unwrap();
        }
    }

    loop {
        continue;
    }
}
