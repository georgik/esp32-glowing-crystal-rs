#![no_std]
#![no_main]

use hal::{
    clock::ClockControl,
    peripherals,
    prelude::*,
    rmt::Rmt,
    rng::Rng,
    Delay,
    IO,
    clock::CpuClock
};
use esp_backtrace as _;
use esp_hal_smartled::{smartLedAdapter, SmartLedsAdapter};
use smart_leds::{
    brightness,
    gamma,
    hsv::{hsv2rgb, Hsv},
    SmartLedsWrite,
};

// Modes
enum Mode {
    Rainbow,
    Candle,
    Flame,
}


#[entry]
fn main() -> ! {
    let peripherals = peripherals::Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock80MHz).freeze();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let button = io.pins.gpio9.into_pull_up_input();
    let mut rng = Rng::new(peripherals.RNG);
    let mut current_mode = Mode::Rainbow;

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

    let flame_color = Hsv {
        hue: 25,
        sat: 240,
        val: 180,
    };


    let mut data;
    let mut data_strip;
    let mut rainbow_hue = 0;

    loop {
        // Check button press to change mode
        if button.is_low().unwrap() {
            current_mode = match current_mode {
                Mode::Rainbow => Mode::Candle,
                Mode::Candle => Mode::Flame,
                Mode::Flame => Mode::Rainbow,
            };
            // Debouncing delay
            delay.delay_ms(1000u16);
        }

        match current_mode {
            Mode::Rainbow => {
                // Iterate over the rainbow!
                rainbow_hue = rainbow_hue + 1;
                if rainbow_hue > 254 {
                    rainbow_hue = 0;
                }
                color.hue = rainbow_hue;
                let value = hsv2rgb(color);
                data = [value];
                data_strip = [value, value, value, value];

                led.write(brightness(gamma(data.iter().cloned()), 60))
                .unwrap();

                led_strip.write(brightness(gamma(data_strip.iter().cloned()), 60))
                    .unwrap();
                delay.delay_ms(250u8);
            },
            Mode::Candle => {
                // Base brightness and flicker mask
                let base_brightness: u8 = 180; // Adjust as needed
                let flicker_mask: u8 = 0x1F; // Mask for lower 5 bits

                // Generate a flicker within the range using bitmask
                let mut buf = [0u8; 1];
                rng.read(&mut buf).unwrap();
                let flicker = (buf[0] & flicker_mask).wrapping_sub(16); // Use wrapping_sub for overflow handling

                // Calculate new brightness with overflow handling
                let new_brightness = base_brightness.wrapping_add(flicker);

                // Apply the new brightness
                let color = hsv2rgb(Hsv {
                    hue: flame_color.hue,
                    sat: 255,
                    val: new_brightness,
                });
                let data = [color];
                let data_strip = [color, color, color, color];

                led.write(brightness(gamma(data.iter().cloned()), 60))
                    .unwrap();

                led_strip.write(brightness(gamma(data_strip.iter().cloned()), 255))
                    .unwrap();

                // Randomized delay
                rng.read(&mut buf).unwrap();
                let delay_time = (buf[0] & 0x3F) + 50;
                delay.delay_ms(delay_time as u8);
            },

            Mode::Flame => {
            }
        }
    }
}
