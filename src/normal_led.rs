use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::PinDriver;
use std::thread::sleep;
use std::time::Duration;

/**
 * Let the LED blink slowly
 * aka vorweihnachtlich
 */
pub fn blink_slow(pins: &mut esp_idf_hal::gpio::Pins) {
    let mut led_pin = PinDriver::output(&mut pins.gpio8).unwrap();
    let mut led_pin2 = PinDriver::output(&mut pins.gpio10).unwrap();

    led_pin.set_low().unwrap();
    led_pin2.set_low().unwrap();
    esp_println::println!("LED ON");
    FreeRtos::delay_ms(1000);

    led_pin.set_high().unwrap();
    led_pin2.set_high().unwrap();
    esp_println::println!("LED OFF");
    FreeRtos::delay_ms(1000);
}

/**
 * Let the LED light up
 */
pub fn static_light(pins: &mut esp_idf_hal::gpio::Pins) {
    let mut led_pin = PinDriver::output(&mut pins.gpio10).unwrap();

    loop {
        led_pin.set_low().unwrap(); // Ensure LED2 is on
        // Sleep to reduce CPU usage
        sleep(Duration::from_millis(100));
    }
}

/**
* Let the LED blink fast
* aka party hart
*/
pub fn blink_fast(pins: &mut esp_idf_hal::gpio::Pins) {
    let mut led_pin = PinDriver::output(&mut pins.gpio8).unwrap();
    let mut led_pin2 = PinDriver::output(&mut pins.gpio10).unwrap();

    loop {
        led_pin.set_low().unwrap();
        led_pin2.set_low().unwrap();
        println!("LED ON");
        FreeRtos::delay_ms(100);

        led_pin.set_high().unwrap();
        led_pin2.set_high().unwrap();
        println!("LED OFF");
        FreeRtos::delay_ms(100);
    }
}