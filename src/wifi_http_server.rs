use crate::{normal_led, smart_led};
use anyhow::Error;
use dotenv::dotenv;
use embedded_svc::http::Method;
use embedded_svc::wifi::{ClientConfiguration, Configuration};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::http::server::{Configuration as HttpServerConfig, EspHttpServer};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::EspWifi;

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

/**
 * Initialize the wifi connection
 * The LED will blink slowly until the connection is established
 */
pub fn init_wifi(peripherals: &mut Peripherals, sys_loop: EspSystemEventLoop) -> anyhow::Result<(), Error> {
    // Reads the .env file
    // https://dev.to/francescoxx/3-ways-to-use-environment-variables-in-rust-4eaf
    dotenv().ok();

    let wlan_ssid = "WLAN-Zingst";
    let wlan_password = "7547112874489301";

    let nvs = EspDefaultNvsPartition::take()?;
    let pins = &mut peripherals.pins;
    let rmt = &mut peripherals.rmt;
    let modem = &mut peripherals.modem;

    let mut wifi_driver = EspWifi::new(
        modem,
        sys_loop,
        Some(nvs),
    )?;

    // Define SSID and Password with appropriate sizes
    let mut ssid: heapless::String<32> = heapless::String::new();
    let mut password: heapless::String<64> = heapless::String::new();

    ssid.push_str(wlan_ssid).unwrap();
    password.push_str(wlan_password).unwrap();

    wifi_driver.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid,
        password,
        ..Default::default()
    }))?;

    wifi_driver.start()?;
    wifi_driver.connect()?;
    while !wifi_driver.is_connected()? {
        let config = wifi_driver.get_configuration()?;
        esp_println::println!("Waiting for station {:?}", config);
        normal_led::blink_slow(pins);
    }

    esp_println::println!("Should be connected");
    esp_println::println!("IP info: {:?}", wifi_driver.sta_netif().get_ip_info()?);

    // Initialize the HTTP server
    let mut httpserver = EspHttpServer::new(&HttpServerConfig::default())
        .expect("Failed to initialize HTTP server");

    // Define Server Request Handler Behaviour on Get for Root URL
    httpserver.fn_handler("/", Method::Get, |request| {
        esp_println::println!("Received HTTP request for /");
        let html = index_html();
        let mut response = request.into_ok_response()?;
        response.write(html.as_bytes())?;
        Ok::<(), Error>(())
    })?;

    // Now call static_light_smart with a mutable reference to peripherals
    // smart_led::set_led_color(pins, rmt, 8, smart_led::Rgb::new(255, 0, 0))?;
    smart_led::rainbow_led_color(pins, rmt, 8)?;
    // normal_led::static_light(&mut peripherals.pins);

    // Keep the function alive
    loop {
        // Main loop keeps the HTTP server and Wi-Fi alive
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}
