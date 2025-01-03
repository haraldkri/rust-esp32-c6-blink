extern crate dotenv;
#[macro_use]
extern crate dotenv_codegen;
mod normal_led;
mod wifi_http_server;
mod smart_led;

use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
// If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_println::println;

/**
 * Main function to start the application
 */
fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    // Peripherals is a singleton
    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take();

    println!("Starting Christmas Hackathon\nThis application is a basic xmas led starter for christmas led blinking.\n");

    wifi_http_server::init_wifi(peripherals, sys_loop.clone().unwrap()).expect("Failed to initialize wifi");
}