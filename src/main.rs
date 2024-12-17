mod normal_led;
mod wifi_http_server;
mod smart_led;

use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
// If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_println::println;

use dotenv::dotenv;
use esp_idf_svc::eventloop::EspSystemEventLoop;

/**
 * Main function to start the application
 */
fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    dotenv().ok();

    // Peripherals is a singleton, so we have to pass a pointer to other functions instead of the instance itself
    // Furthermore it needs to be mutable for the functions to access things inside the struct (like pins and modem)
    let mut peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take();

    println!("Starting Christmas Hackathon\nThis application is a basic xmas led starter for christmas led blinking.\n");


    wifi_http_server::init_wifi(peripherals, sys_loop.clone().unwrap()).expect("Failed to initialize wifi");
}