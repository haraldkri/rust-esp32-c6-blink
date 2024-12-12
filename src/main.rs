use esp_idf_hal::{delay::FreeRtos, gpio::PinDriver, peripherals::Peripherals};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_println::println;

use esp_idf_svc::{
    wifi::EspWifi,
    nvs::EspDefaultNvsPartition,
    eventloop::EspSystemEventLoop,
};
use embedded_svc::wifi::{ClientConfiguration, Wifi, Configuration};
use heapless::String;

/**
 * Initialize the wifi connection
 * The LED will blink slowly until the connection is established
 */
fn init_wifi(peripherals: &mut Peripherals) {
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut wifi_driver = EspWifi::new(
        &mut peripherals.modem,
        sys_loop,
        Some(nvs),
    ).unwrap();

    // Define SSID and Password with appropriate sizes
    let mut ssid: String<32> = String::new();
    let mut password: String<64> = String::new();

    ssid.push_str("WIFI_SSID").unwrap();
    password.push_str("WIFI_PASSWORD").unwrap();

    wifi_driver.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid,
        password: password,
        ..Default::default()
    })).unwrap();

    wifi_driver.start().unwrap();
    wifi_driver.connect().unwrap();
    while !wifi_driver.is_connected().unwrap() {
        let config = wifi_driver.get_configuration().unwrap();
        println!("Waiting for station {:?}", config);
        blink_slow(&mut peripherals.pins);
    }

    println!("Should be connected");
    println!("IP info: {:?}", wifi_driver.sta_netif().get_ip_info().unwrap());
}

/**
 * Let the LED blink slowly
 * aka vorweihnachtlich
 */
fn blink_slow(pins: &mut esp_idf_hal::gpio::Pins) {
    let mut led_pin = PinDriver::output(&mut pins.gpio8).unwrap();
    let mut led_pin2 = PinDriver::output(&mut pins.gpio10).unwrap();

    led_pin.set_low().unwrap();
    led_pin2.set_low().unwrap();
    println!("LED ON");
    FreeRtos::delay_ms(1000);

    led_pin.set_high().unwrap();
    led_pin2.set_high().unwrap();
    println!("LED OFF");
    FreeRtos::delay_ms(1000);
}

/**
 * Let the LED blink fast
 * aka party hart
 */
fn blink_fast(pins: &mut esp_idf_hal::gpio::Pins) {
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

/**
 * Main function to start the application
 */
fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    // Peripherals is a singleton, so we have to pass a pointer to other functions instead of the instance itself
    // Furthermore it needs to be mutable for the functions to access things inside the struct (like pins and modem)
    let mut peripherals = Peripherals::take().unwrap();

    println!("Starting Christmas Hackathon\nThis application is a basic xmas led starter for christmas led blinking.\n");


    init_wifi(&mut peripherals);
    blink_fast(&mut peripherals.pins);
}