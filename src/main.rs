#![no_main]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate panic_halt;

extern crate stm32f042_hal as hal;

extern crate shared_bus;

extern crate ina260;
extern crate numtoa;
extern crate ssd1306;

use cortex_m_rt::entry;

use ina260::INA260;
use ssd1306::mode::TerminalMode;
use ssd1306::Builder;

use numtoa::NumToA;

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

        /* Get delay provider */
        let mut delay = Delay::new(cp.SYST, clocks);

        let mut led = gpioa.pa1.into_push_pull_output();
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

        /* Setup I2C1 */
        let mut i2c = I2c::i2c1(p.I2C1, (scl, sda), 400.khz());

        let i2c_bus = shared_bus::CortexMBusManager::new(i2c);

        let mut disp: TerminalMode<_> = Builder::new()
            .with_i2c_addr(0x3c)
            .connect_i2c(i2c_bus.acquire())
            .into();
        delay.delay_ms(300_u16);
        led.set_low();
        delay.delay_ms(300_u16);
        led.set_high();

        disp.init().unwrap();

        let _ = disp.clear();
        let _ = disp.write_str("Initialising INA260 at I2C address 0x40...\n\r");
        delay.delay_ms(300_u16);

        let mut ina260 = INA260::new(i2c_bus.acquire(), 0x40).unwrap();
        let _ = disp.clear();

        /* Endless loop */
        loop {
            /* Read voltage */
            let voltage = ina260.voltage().unwrap();

            /* Read current */
            let current = ina260.current().unwrap();

            /* Read power */
            let power = ina260.power().unwrap();

            let _ = write!(
                disp,
                "U: {:2}.{:04} V    ",
                voltage / 1000000,
                (voltage % 1000000) / 100
            );

            let _ = write!(
                disp,
                "I: {:2}.{:04} A    ",
                current / 1000000,
                (current.abs() % 1000000) / 100
            );

            let _ = write!(
                disp,
                "P: {:2}.{:04} W    ",
                power / 1000000,
                (power % 1000000) / 100
            );

            for _ in 0..5 * 16 {
                let _ = disp.print_char(' ');
            }

            led.set_low();
            delay.delay_ms(50_u16);
            led.set_high();
        }
    }

    loop {}
}
