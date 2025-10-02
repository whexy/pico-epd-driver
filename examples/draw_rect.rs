//! Test partial screen updating feature
#![no_std]
#![no_main]

extern crate alloc;
use alloc::boxed::Box;
use alloc_cortex_m::CortexMHeap;

use bitvec::order::Msb0;
use bitvec::slice::BitSlice;
use bitvec::view::BitView;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::spi::{Config as SpiConfig, Phase, Polarity, Spi};
use panic_probe as _;

use pico_epd_driver::epd_driver::{Epd800x480, EpdBus, HEIGHT, WIDTH};
use pico_epd_driver::ui::pack_bitmap;

const RESOLUTION: usize = WIDTH * HEIGHT;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let heap_start = cortex_m_rt::heap_start() as usize;
    unsafe extern "C" {
        static _heap_ceiling: u8;
    }
    let heap_end = core::ptr::addr_of!(_heap_ceiling) as usize;

    unsafe { ALLOCATOR.init(heap_start, heap_end - heap_start) }

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
    let led = Output::new(p.PIN_25, Level::High);

    // Initialize EPD
    let bus = EpdBus::new(spi, cs, dc, rst, busy);
    let mut epd = Epd800x480::new(bus, led);
    if epd.init().await.is_err() {
        panic!();
    }

    if epd.clear().await.is_err() {
        panic!();
    }

    {
        // Draw a chessboard pattern on full screen
        let mut view = Box::new([0u8; RESOLUTION / 8]);
        let cb_bits: &mut BitSlice<u8, Msb0> = view.view_bits_mut::<Msb0>();
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                let idx = y * WIDTH + x;
                if ((x / 50) + (y / 50)) % 2 == 0 {
                    cb_bits.set(idx, true);
                } else {
                    cb_bits.set(idx, false);
                }
            }
        }

        let (_, fb) = pack_bitmap(cb_bits, 0, 0, WIDTH, HEIGHT).expect("Failed to pack bitmap");

        if epd.display(&fb).await.is_err() {
            panic!();
        }
    }

    {
        // Draw a smaller black rectangle in the center
        let mut view = Box::new([0u8; 200 * 200 / 8]);
        let cb_bits = view.view_bits_mut::<Msb0>();
        for x in 0..200 {
            for y in 0..200 {
                let idx = y * 200 + x;
                if ((x / 20) + (y / 20)) % 2 == 0 {
                    cb_bits.set(idx, true);
                } else {
                    cb_bits.set(idx, false);
                }
            }
        }
        let (rect, fb) = pack_bitmap(cb_bits, 50, 50, 200, 200).expect("Failed to pack bitmap");
        if epd.display_partial(&fb, rect).await.is_err() {
            panic!();
        }
    }

    if epd.sleep().await.is_err() {
        panic!();
    }

    loop {
        epd.flash_led().await;
    }
}
