#![no_std]
#![no_main]

use hal::{clock::ClockControl, peripherals, prelude::*, rmt::Rmt, Delay, IO,
clock::CpuClock};
use esp_backtrace as _;
use esp_hal_smartled::{smartLedAdapter, SmartLedsAdapter};
use smart_leds::{
    brightness,
    gamma,
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite,
};

#[entry]
fn main() -> ! {
    let peripherals = peripherals::Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock80MHz).freeze();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let rmt = Rmt::new(peripherals.RMT, 80u32.MHz(), &clocks).unwrap();

    // On-board LED connected to pin 2
    let mut led = <smartLedAdapter!(0, 1)>::new(rmt.channel0, io.pins.gpio2);

    // LED strip connected to pin 6 with 4 LEDs
    let mut led_strip = <smartLedAdapter!(1, 4)>::new(rmt.channel1, io.pins.gpio6);

    let mut delay = Delay::new(&clocks);

    let mut color = Hsv {
        hue: 0,
        sat: 255,
        val: 255,
    };

    let mut data;
    let mut data_strip;

    loop {
        // Iterate over the rainbow!
        for hue in 0..=255 {
            color.hue = hue;
            let value = hsv2rgb(color);
            data = [value];
            data_strip = [value, value, value, value];

            led.write(brightness(gamma(data.iter().cloned()), 60))
            .unwrap();

            led_strip.write(brightness(gamma(data_strip.iter().cloned()), 60))
                .unwrap();
            delay.delay_ms(250u8);
        }
    }
}
