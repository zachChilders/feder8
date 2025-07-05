use std::thread;
use std::time::Duration;

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_hal::gpio::*;
use esp_idf_hal::delay::FreeRtos;
use log::*;

fn main() -> anyhow::Result<()> {
    // Initialize the logger
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    println!("Hello, ESP32 World!");
    info!("ESP-IDF Rust Hello World starting up...");

    // Take the peripherals
    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    // Configure the onboard LED (GPIO2 on most ESP32 boards)
    let mut led = PinDriver::output(pins.gpio2)?;

    info!("LED pin configured successfully");

    let mut counter = 0;
    loop {
        counter += 1;
        
        // Turn LED on
        led.set_high()?;
        println!("LED ON  - Counter: {}", counter);
        info!("LED ON  - Counter: {}", counter);
        
        // Wait 1 second
        FreeRtos::delay_ms(1000);
        
        // Turn LED off
        led.set_low()?;
        println!("LED OFF - Counter: {}", counter);
        info!("LED OFF - Counter: {}", counter);
        
        // Wait 1 second
        FreeRtos::delay_ms(1000);
        
        // Print a hello message every 10 cycles
        if counter % 10 == 0 {
            println!("Hello from ESP32! Running for {} cycles", counter);
            info!("Hello from ESP32! Running for {} cycles", counter);
        }
    }
}