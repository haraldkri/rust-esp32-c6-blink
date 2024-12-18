use crate::smart_led;
use crate::smart_led::{hex_to_rgb, set_led_color, set_led_colors};
use anyhow::Error;
use embedded_svc::http::Method;
use embedded_svc::wifi::{ClientConfiguration, Configuration};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::http::server::{Configuration as HttpServerConfig, EspHttpServer};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::EspWifi;
use serde::Deserialize;
use std::cell::RefCell;
use std::thread::sleep;
use std::time::Duration;

// Define a struct to match the expected JSON format
#[derive(Deserialize)]
struct ColorRequest {
    leds: Vec<String>, // Expect an array of hex color strings
}

pub fn index_html() -> std::string::String {
    r#"
<!DOCTYPE html>
<html>
    <head>
        <meta charset="utf-8">
        <title>esp-rs web server</title>
    </head>
    <body>
    Hello World from ESP!
    </body>
</html>
"#.to_string()
}

/// Initializes the Wi-Fi connection using the provided peripherals and system event loop.
/// This function retrieves SSID and password from environment variables, attempts to connect
/// to the specified Wi-Fi network, and indicates connection status by controlling an LED.
/// It also sets up an HTTP server with handlers for setting and getting LED colors.
///
/// The function performs the following steps:
/// 1. Reads the Wi-Fi credentials from environment variables.
/// 2. Configures the Wi-Fi driver with the credentials.
/// 3. If not connected, sets the LED to yellow and tries to connect to the Wi-Fi network.
/// 4. Once connected, sets the LED to green and logs the connection status.
/// 5. Initializes an HTTP server with routes for color setting and retrieval.
/// 6. Enters an infinite loop, keeping the main thread alive.
pub fn init_wifi(peripherals: Peripherals, sys_loop: EspSystemEventLoop) -> anyhow::Result<(), Error> {
    // Read SSID and password from the environment
    let wlan_ssid = dotenv!("WIFI_SSID");
    let wlan_password = dotenv!("WIFI_PASSWORD");

    let mut ssid: heapless::String<32> = heapless::String::new();
    let mut password: heapless::String<64> = heapless::String::new();

    ssid.push_str(&*wlan_ssid).unwrap();
    password.push_str(&*wlan_password).unwrap();

    // Wrap the LED pin in a RefCell to allow interior mutability
    let led_pin = RefCell::new(peripherals.pins.gpio8);
    let rmt_channel = RefCell::new(peripherals.rmt.channel0);

    let nvs = EspDefaultNvsPartition::take()?;
    let mut wifi_driver = EspWifi::new(peripherals.modem, sys_loop, Some(nvs))?;

    wifi_driver.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid,
        password,
        ..Default::default()
    }))?;

    if (!wifi_driver.is_connected()?) {
        let mut pin = led_pin.borrow_mut();
        let mut channel = rmt_channel.borrow_mut();
        let _ = set_led_color(&mut *pin, &mut *channel, smart_led::Rgb::new(255, 255, 0));
    }

    wifi_driver.start()?;
    wifi_driver.connect()?;
    while !wifi_driver.is_connected()? {
        esp_println::println!("Connecting...");
        // Sleep to reduce CPU usage
        sleep(Duration::from_millis(200));
    }

    if (wifi_driver.is_connected()?) {
        let mut pin = led_pin.borrow_mut();
        let mut channel = rmt_channel.borrow_mut();
        let _ = set_led_color(&mut *pin, &mut *channel, smart_led::Rgb::new(0, 255, 0));
    }

    esp_println::println!("Wi-Fi Connected!");

    let mut httpserver = EspHttpServer::new(&HttpServerConfig::default())?;


    use std::sync::{Arc, Mutex};

    // Shared state to hold the current LED colors
    let led_colors = Arc::new(Mutex::new(vec![String::from("00FF00")]));

    // Clone Arc to move into POST handler
    let led_colors_post = Arc::clone(&led_colors);
    httpserver.fn_handler("/set_color", Method::Post, move |mut request| {
        esp_println::println!("GET /set_color handler invoked");
        let mut body = [0u8; 512];
        let length = request.read(&mut body)?;
        let body_str = std::str::from_utf8(&body[..length])?;

        let color_request: ColorRequest = serde_json::from_str(body_str)
            .map_err(|e| {
                esp_println::println!("Failed to parse JSON: {}", e);
                Error::msg("Invalid JSON payload")
            })?;

        // Update shared state outside of critical section
        let color_data = {
            let mut current_colors = led_colors_post.lock().map_err(|_| Error::msg("Mutex poisoned"))?;
            current_colors.clear();
            current_colors.extend(color_request.leds.clone());
            current_colors.clone()
        };

        // Perform LED updates
        let mut rgb_values = Vec::new();
        for hex_color in color_data {
            let rgb = hex_to_rgb(&hex_color).map_err(|e| {
                esp_println::println!("Failed to convert hex to RGB: {}", e);
                Error::msg("Invalid hex color")
            })?;

            // Add the RGB values to the vector
            rgb_values.push(smart_led::Rgb::new(rgb.r, rgb.g, rgb.b));
        }

        for (index, rgb) in rgb_values.iter().enumerate() {
            println!("led_{}: R: {}, G: {}, B: {}", index, rgb.r, rgb.g, rgb.b);
        }

        let mut pin = led_pin.borrow_mut();
        let mut channel = rmt_channel.borrow_mut();

        set_led_colors(&mut *pin, &mut *channel, &*rgb_values)?;


        let mut response = request.into_ok_response()?;
        response.write(b"{\"status\": \"success\"}")?;
        Ok::<(), Error>(())
    })?;


    // Clone Arc to move into GET handler
    let led_colors_get = Arc::clone(&led_colors);
    httpserver.fn_handler("/get_color", Method::Get, move |request| {
        esp_println::println!("GET /get_color handler invoked");
        let current_colors = led_colors_get.lock().map_err(|_| Error::msg("Mutex poisoned"))?;

        let response_body = serde_json::to_string(&*current_colors).map_err(|e| {
            esp_println::println!("Failed to serialize colors: {}", e);
            Error::msg("Serialization error")
        })?;

        let mut response = request.into_ok_response()?;
        response.write(response_body.as_bytes())?;
        Ok::<(), Error>(())
    })?;

    httpserver.fn_handler("/", Method::Get, |request| {
        esp_println::println!("GET / handler invoked");
        let html = index_html();
        let mut response = request.into_ok_response()?;
        response.write(html.as_bytes())?;
        Ok::<(), Error>(())
    })?;


    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}


