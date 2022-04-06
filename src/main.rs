#![no_main]
#![no_std]

use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::text::Text;
use panic_halt as _;
use ssd1306::prelude::*;
use ssd1306::size::DisplaySize128x64;
use ssd1306::Ssd1306;

use stm32f0xx_hal as hal;

use heapless::String;
use ssd1306::I2CDisplayInterface;

use cortex_m_rt::entry;

use embedded_graphics::{mono_font::ascii::FONT_9X15, pixelcolor::BinaryColor, prelude::*};
use ina260::INA260;

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

            let i2c_bus = shared_bus::BusManagerSimple::new(i2c);

            let text_style = MonoTextStyleBuilder::new()
                .font(&FONT_9X15)
                .text_color(BinaryColor::On)
                .build();

            let interface = I2CDisplayInterface::new(i2c_bus.acquire_i2c());
            let mut disp = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
                .into_buffered_graphics_mode();
            disp.init().map_err(drop).unwrap();
            disp.flush().map_err(drop).unwrap();

            let mut ina260 = INA260::new(i2c_bus.acquire_i2c(), 0x40)
                .map_err(drop)
                .unwrap();

            // Slow down sampling a bit for more accuracy
            ina260
                .set_bvconvtime_mode(ina260::BVConvTime::MS4_156)
                .map_err(drop)
                .unwrap();
            ina260
                .set_scconvtime_mode(ina260::SCConvTime::MS4_156)
                .map_err(drop)
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

                // Read voltage current and power
                let voltage = ina260.voltage_split().map_err(drop).unwrap();
                let current = ina260.current_split().map_err(drop).unwrap();
                let power = ina260.power_split().map_err(drop).unwrap();

                let mut s: String<32> = String::new();
                write!(
                    s,
                    "{:3}.{:05}V\n{:3}.{:05}A\n{:3}.{:05}W",
                    voltage.0, voltage.1, current.0, current.1, power.0, power.1
                )
                .map_err(drop)
                .ok();

                Text::with_baseline(
                    s.as_str(),
                    Point::zero(),
                    text_style,
                    embedded_graphics::text::Baseline::Top,
                )
                .draw(&mut disp)
                .ok();

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
