#![no_std]
#![no_main]

use esp_hal::{
    clock::ClockControl, clock::CpuClock, delay::Delay, gpio::Io, peripherals::Peripherals,
    prelude::*, rmt::Rmt, rng::Rng, system::SystemControl,
};

use esp_backtrace as _;
use esp_hal_smartled::{smartLedBuffer, SmartLedsAdapter};
use esp_println::println;
use smart_leds::{
    brightness, gamma,
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
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock80MHz).freeze();
    let delay = Delay::new(&clocks);

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let button = io.pins.gpio9;
    let mut rng = Rng::new(peripherals.RNG);
    let mut current_mode = Mode::Rainbow;

    let rmt = Rmt::new(peripherals.RMT, 80u32.MHz(), &clocks).unwrap();

    let on_board_led_pin = io.pins.gpio2;
    let led_strip_pin = io.pins.gpio6;

    // On-board LED connected to pin 2
    let rmt_buffer_on_board_led = smartLedBuffer!(1);
    let rmt_buffer_led_strip = smartLedBuffer!(4);
    let mut led = SmartLedsAdapter::new(
        rmt.channel0,
        on_board_led_pin,
        rmt_buffer_on_board_led,
        &clocks,
    );
    let mut led_strip =
        SmartLedsAdapter::new(rmt.channel1, led_strip_pin, rmt_buffer_led_strip, &clocks);

    let delay = Delay::new(&clocks);

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

    println!("Entering main loop...");

    loop {
        // Check button press to change mode
        if button.is_low() {
            current_mode = match current_mode {
                Mode::Rainbow => Mode::Candle,
                Mode::Candle => Mode::Flame,
                Mode::Flame => Mode::Rainbow,
            };
            // Debouncing delay
            delay.delay_millis(1000u32);
            println!("Change mode");
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

                led_strip
                    .write(brightness(gamma(data_strip.iter().cloned()), 60))
                    .unwrap();
                delay.delay_millis(250u32);
            }
            Mode::Candle => {
                // Base brightness and flicker mask
                let base_brightness: u8 = 180; // Adjust as needed
                let flicker_mask: u8 = 0x1F; // Mask for lower 5 bits

                // Generate a flicker within the range using bitmask
                let mut buf = [0u8; 1];
                rng.read(&mut buf);
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

                led_strip
                    .write(brightness(gamma(data_strip.iter().cloned()), 255))
                    .unwrap();

                // Randomized delay
                rng.read(&mut buf);
                let delay_time = (buf[0] & 0x3F) + 50;
                delay.delay_millis(delay_time as u32);
            }

            Mode::Flame => {}
        }
    }
}
