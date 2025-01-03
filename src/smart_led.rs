use crate::smart_led;
use anyhow::{bail, Error, Result};
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::rmt::config::TransmitConfig;
use esp_idf_hal::rmt::{FixedLengthSignal, PinState, Pulse, TxRmtDriver, CHANNEL0};
use serde::Deserialize;
use std::time::Duration;

/// Function to control a static light on the smart LED. It first sets the LED to white for 3 seconds,
/// then enters an infinite loop cycling through rainbow colors at 20% brightness.
pub fn static_light_smart(peripherals: &mut Peripherals) {
    let led = &mut peripherals.pins.gpio8;
    let channel = &mut peripherals.rmt.channel0;
    let config = TransmitConfig::new().clock_divider(1);
    let mut tx = TxRmtDriver::new(channel, led, &config).expect("Failed to create RMT driver");

    // 3 seconds white at 10% brightness
    neopixel(Rgb::new(25, 25, 25), &mut tx).expect("Failed to set neopixel color");
    FreeRtos::delay_ms(3000);

    // infinite rainbow loop at 20% brightness
    (0..360).cycle().try_for_each(|hue| {
        FreeRtos::delay_ms(10);
        let rgb = Rgb::from_hsv(hue, 100, 20)?;
        neopixel(rgb, &mut tx)
    }).expect("Failed to set neopixel color");
}

/// Function to control the rainbow LED color. It takes the pin number as input, sets the LED to white
/// for 3 seconds, then cycles through rainbow colors at 20% brightness.
pub fn rainbow_led_color(pins: &mut esp_idf_hal::gpio::Pins, rmt: &mut esp_idf_hal::rmt::RMT, pin_number: u32) -> Result<(), Error> {
    let pin = match pin_number {
        8 => &mut pins.gpio8,
        // Add more GPIO pins as needed
        _ => bail!("Invalid GPIO pin number: {}", pin_number),
    };

    let channel = &mut rmt.channel0;
    let config = TransmitConfig::new().clock_divider(1);
    let mut tx = TxRmtDriver::new(channel, pin, &config)?;

    // 3 seconds white at 10% brightness
    neopixel(Rgb::new(25, 25, 25), &mut tx).expect("Failed to set neopixel color");
    FreeRtos::delay_ms(3000);

    // infinite rainbow loop at 20% brightness
    (0..360).cycle().try_for_each(|hue| {
        FreeRtos::delay_ms(10);
        let rgb = Rgb::from_hsv(hue, 100, 20)?;
        neopixel(rgb, &mut tx)
    }).expect("Failed to set neopixel color");

    Ok(())
}

/// Function to set the LED color using the specified RGB value. It initializes the RMT driver and sends
/// the RGB value to the LED.
pub fn set_led_color(pin: &mut esp_idf_hal::gpio::Gpio8, channel: &mut CHANNEL0, rgb: Rgb) -> Result<(), Error> {
    println!("LED ON - r{},g{},b{}", rgb.r, rgb.g, rgb.b);

    let config = TransmitConfig::new().clock_divider(1);
    let mut tx = TxRmtDriver::new(channel, pin, &config)?;

    // Set the LED to the specified RGB color
    neopixel(rgb, &mut tx)?;
    Ok(())
}

/// Function to send a color to a NeoPixel LED. It constructs the necessary signal and sends it
/// to the LED driver.
pub fn neopixel(rgb: Rgb, tx: &mut TxRmtDriver) -> Result<(), Error> {
    let color: u32 = rgb.into();
    let ticks_hz = tx.counter_clock()?;
    let (t0h, t0l, t1h, t1l) = (
        Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(350))?,
        Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(800))?,
        Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(700))?,
        Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(600))?,
    );
    let mut signal = FixedLengthSignal::<24>::new();
    for i in (0..24).rev() {
        let p = 2_u32.pow(i);
        let bit: bool = p & color != 0;
        let (high_pulse, low_pulse) = if bit { (t1h, t1l) } else { (t0h, t0l) };
        signal.set(23 - i as usize, &(high_pulse, low_pulse))?;
    }
    tx.start_blocking(&signal)?;
    Ok(())
}

/// Function to set multiple LED colors in a chain. It takes an array of RGB values and sends them
/// to the LED chain using the RMT driver.
pub fn set_led_colors(
    pin: &mut esp_idf_hal::gpio::Gpio8,
    channel: &mut CHANNEL0,
    colors: &[Rgb],
) -> Result<(), Error> {
    println!("Setting LED chain colors: {:?}", colors);

    let config = TransmitConfig::new().clock_divider(1);
    let mut tx = TxRmtDriver::new(channel, pin, &config)?;

    // Send the colors to the LED chain
    if let Err(e) = neopixel_chain(colors, &mut tx) {
        esp_println::println!("Error setting LED colors: {:?}", e);
        return Err(e);
    }
    Ok(())
}

/// Function to send an array of RGB colors to a NeoPixel LED chain. It constructs the necessary signal
/// and sends it to the LED driver.
pub fn neopixel_chain(colors: &[Rgb], tx: &mut TxRmtDriver) -> Result<(), Error> {
    let ticks_hz = tx.counter_clock()?;
    let (t0h, t0l, t1h, t1l) = (
        Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(350))?,
        Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(800))?,
        Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(700))?,
        Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(600))?,
    );

    /// Create a signal for the entire LED chain
    /// 24 bits per LED, 21 LEDs -> 24 * 21 = 504 Bytes used for the signal, make sure the current CONFIG_ESP_MAIN_TASK_STACK_SIZE is big enough to handle that
    let mut signal = FixedLengthSignal::<{ 24 * 21 }>::new(); // Adjust for max LED count
    for (index, rgb) in colors.iter().enumerate() {
        let color = rgb.to_u32(); // Use the helper function
        for i in (0..24).rev() {
            let bit = (color & (1 << i)) != 0;
            let (high_pulse, low_pulse) = if bit { (t1h, t1l) } else { (t0h, t0l) };
            signal.set(index * 24 + (23 - i as usize), &(high_pulse, low_pulse))?;
        }
    }

    // Send the signal
    tx.start_blocking(&signal)?;
    Ok(())
}

/// Structure representing an RGB color with red, green, and blue components.
#[derive(Debug)]
pub(crate) struct Rgb {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
}

/// Structure to hold an array of RGB values.
pub(crate) struct RgbArr {
    pub(crate) rgb_values: Vec<Rgb>, // An array of RGB values
}

/// Structure to deserialize the RGB array from JSON.
#[derive(Deserialize)]
pub(crate) struct RgbValueArray {
    pub(crate) rgb_values: Vec<RgbValue>, // An array of RGB values
}

/// Structure representing RGB values for deserialization.
#[derive(Deserialize)]
pub struct RgbValue {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
}

impl Rgb {
    /// Constructor to create a new RGB color with the specified red, green, and blue values.
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Helper function to convert Rgb into u32 in GRB format.
    pub fn to_u32(&self) -> u32 {
        ((self.g as u32) << 16) | ((self.r as u32) << 8) | (self.b as u32)
    }

    /// Converts hue, saturation, value to RGB.
    pub fn from_hsv(h: u32, s: u32, v: u32) -> Result<Rgb, Error> {
        let h = h % 360;
        let s = s.clamp(0, 100) as f32 / 100.0;
        let v = v.clamp(0, 100) as f32 / 100.0;

        let c = v * s;
        let x = c * (1.0 - ((h as f32 / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r, g, b) = match h {
            0..=59 => (c, x, 0.0),
            60..=119 => (x, c, 0.0),
            120..=179 => (0.0, c, x),
            180..=239 => (0.0, x, c),
            240..=299 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Ok(Rgb::new(((r + m) * 255.0) as u8, ((g + m) * 255.0) as u8, ((b + m) * 255.0) as u8))
    }
}


impl From<Rgb> for u32 {
    /// Convert RGB to u32 color value
    ///
    /// e.g. rgb: (1,2,4)
    /// G        R        B
    /// 7      0 7      0 7      0
    /// 00000010 00000001 00000100
    fn from(rgb: Rgb) -> Self {
        ((rgb.g as u32) << 16) | ((rgb.r as u32) << 8) | rgb.b as u32
    }
}

/// Convert a hex color string to RGB values.
pub fn hex_to_rgb(hex: &str) -> std::result::Result<smart_led::Rgb, &'static str> {
    let hex = hex.trim_start_matches('#'); // Remove the '#' if present
    if hex.len() != 6 {
        return Err("Hex color must be 6 characters long");
    }

    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid hex for red")?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid hex for green")?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid hex for blue")?;

    Ok(smart_led::Rgb::new(r, g, b))
}