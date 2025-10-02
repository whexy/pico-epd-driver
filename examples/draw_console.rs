//! Test the simplified console API

#![no_std]
#![no_main]

extern crate alloc;
use alloc_cortex_m::CortexMHeap;

use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::spi::{Config as SpiConfig, Phase, Polarity, Spi};
use panic_probe as _;

use pico_epd_driver::console::EpdConsole;
use pico_epd_driver::epd_driver::{Epd800x480, EpdBus};

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Initialize SPI for EPD
    let mut spi_config = SpiConfig::default();
    spi_config.frequency = 2_000_000;
    spi_config.polarity = Polarity::IdleLow;
    spi_config.phase = Phase::CaptureOnFirstTransition;

    let spi = Spi::new_txonly(p.SPI0, p.PIN_6, p.PIN_7, p.DMA_CH0, spi_config);

    // EPD control pins
    let cs = Output::new(p.PIN_5, Level::High);
    let dc = Output::new(p.PIN_8, Level::Low);
    let rst = Output::new(p.PIN_9, Level::High);
    let busy = Input::new(p.PIN_10, Pull::None);

    // Initialize EPD
    let bus = EpdBus::new(spi, cs, dc, rst, busy);
    let led = Output::new(p.PIN_25, Level::Low);
    let mut epd = Epd800x480::new(bus, led);
    epd.init().await.expect("Failed to initialize EPD");

    // Initialize console
    let mut console = EpdConsole::new(&mut epd);
    console.show().await.expect("Failed to show console");

    // Test basic push functionality
    console
        .push("Console initialized")
        .await
        .expect("Failed to push");

    // Test messages
    console
        .push("System startup complete")
        .await
        .expect("Failed to push");

    console
        .push("Battery level OK")
        .await
        .expect("Failed to push");

    console
        .push("All sensors connected")
        .await
        .expect("Failed to push");

    // Test partial refresh behavior - add messages one by one
    console
        .push("=== Testing partial refresh ===")
        .await
        .expect("Failed to push");

    for _i in 1..=10 {
        console
            .push("Partial update test")
            .await
            .expect("Failed to push");
    }

    // Test rapid messages
    console
        .push("=== Testing rapid messages ===")
        .await
        .expect("Failed to push");

    for _i in 1..=20 {
        console.push("Rapid message").await.expect("Failed to push");
    }

    // Test hide/show functionality
    console.hide();

    console.show().await.expect("Failed to show console");
    console
        .push("Console visible again!")
        .await
        .expect("Failed to push");
}
