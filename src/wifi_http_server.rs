use crate::smart_led;
use crate::smart_led::set_led_color;
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


pub fn init_wifi(peripherals: Peripherals, sys_loop: EspSystemEventLoop) -> anyhow::Result<(), Error> {
    // Wrap the LED pin in a RefCell to allow interior mutability
    let led_pin = RefCell::new(peripherals.pins.gpio8);
    let rmt_channel = RefCell::new(peripherals.rmt.channel0);

    let wlan_ssid = "WLAN-Zingst";
    let wlan_password = "7547112874489301";

    let mut ssid: heapless::String<32> = heapless::String::new();
    let mut password: heapless::String<64> = heapless::String::new();

    ssid.push_str(wlan_ssid).unwrap();
    password.push_str(wlan_password).unwrap();

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


    // Define Server Request Handler Behaviour on Get for Root URL
    httpserver.fn_handler("/", Method::Get, |request| {
        esp_println::println!("Received HTTP request for /");
        let html = index_html();
        let mut response = request.into_ok_response()?;
        response.write(html.as_bytes())?;
        Ok::<(), Error>(())
    })?;


    // POST /set_color handler
    httpserver.fn_handler("/set_color", Method::Post, move |mut request| {
        let mut body = [0u8; 512];
        let length = request.read(&mut body)?;
        let body_str = std::str::from_utf8(&body[..length])?;

        // Deserialize the incoming JSON payload
        let color_request: ColorRequest = serde_json::from_str(body_str)
            .map_err(|e| {
                esp_println::println!("Failed to parse JSON: {}", e);
                Error::msg("Invalid JSON payload")
            })?;

        // Process each color in the `leds` array
        for hex_color in color_request.leds {
            let rgb = hex_to_rgb(&hex_color).map_err(|e| {
                esp_println::println!("Failed to convert hex to RGB: {}", e);
                Error::msg("Invalid hex color")
            })?;

            esp_println::println!("Setting LED to: R={}, G={}, B={}", rgb.r, rgb.g, rgb.b);
            let mut pin = led_pin.borrow_mut(); // Mutably borrow the GPIO pin
            let mut channel = rmt_channel.borrow_mut(); // Mutably borrow the RMT channel

            let _ = set_led_color(&mut *pin, &mut *channel, smart_led::Rgb::new(rgb.r, rgb.g, rgb.b));
        }

        let mut response = request.into_ok_response()?;
        response.write(b"{\"status\": \"success\"}")?;
        Ok::<(), Error>(())
    })?;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}

/// Convert a hex color string to RGB values.
fn hex_to_rgb(hex: &str) -> Result<smart_led::Rgb, &'static str> {
    let hex = hex.trim_start_matches('#'); // Remove the '#' if present
    if hex.len() != 6 {
        return Err("Hex color must be 6 characters long");
    }

    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid hex for red")?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid hex for green")?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid hex for blue")?;

    Ok(smart_led::Rgb::new(r, g, b))
}
