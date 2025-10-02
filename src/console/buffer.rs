//! Console buffer management - handles history and framebuffer without display logic.

#![allow(clippy::needless_range_loop)]

use core::convert::Infallible;
use embassy_time::Instant;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    mono_font::{MonoTextStyle, ascii::FONT_9X18},
    pixelcolor::{BinaryColor, BinaryColor::Off, BinaryColor::On},
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::{Baseline, Text},
};
use heapless::{String as HString, Vec as HVec};
use static_cell::StaticCell;

use crate::epd_driver::{BUF_SIZE, HEIGHT, WIDTH};

// ---------- Console config ----------
pub const MARGIN: i32 = 9;
pub const LINE_H: i32 = 18; // FONT_9X18
pub const MAX_VISIBLE_LINES: usize = ((HEIGHT as i32 - 2 * MARGIN) / LINE_H) as usize;
pub const HISTORY_CAP: usize = 128;
pub const LINE_CAP: usize = 96;

// Dedicated console framebuffer
static CONSOLE_FB: StaticCell<[u8; BUF_SIZE]> = StaticCell::new();

/// Private MSB-first 1bpp draw target over a byte buffer.
pub struct MonoBuf<'a> {
    buf: &'a mut [u8],
    w: u32,
    h: u32,
}

impl<'a> MonoBuf<'a> {
    pub fn new(buf: &'a mut [u8], w: u32, h: u32) -> Self {
        Self { buf, w, h }
    }

    #[inline]
    fn set_pixel(&mut self, x: u32, y: u32, on: bool) {
        if x >= self.w || y >= self.h {
            return;
        }
        let idx = (y * self.w + x) as usize;
        let byte = idx >> 3;
        let bit = 7 - (idx & 7);
        if on {
            self.buf[byte] |= 1 << bit;
        } else {
            self.buf[byte] &= !(1 << bit);
        }
    }

    fn fill(&mut self, v: u8) {
        for b in self.buf.iter_mut() {
            *b = v;
        }
    }

    pub fn buffer(&self) -> &[u8] {
        self.buf
    }
}

impl OriginDimensions for MonoBuf<'_> {
    fn size(&self) -> Size {
        Size::new(self.w, self.h)
    }
}

impl DrawTarget for MonoBuf<'_> {
    type Color = BinaryColor;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels {
            if let (Ok(x), Ok(y)) = (u32::try_from(coord.x), u32::try_from(coord.y)) {
                self.set_pixel(x, y, matches!(color, On));
            }
        }
        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.fill(if matches!(color, On) { 0xFF } else { 0x00 });
        Ok(())
    }
}

/// Refresh strategy for the console buffer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshStrategy {
    /// Full display refresh needed
    Full,
    /// Partial refresh with number of new lines
    Partial { new_lines: usize },
    /// No refresh needed
    None,
}

/// Console buffer state - manages history and framebuffer without display logic
pub struct ConsoleBuffer<'a> {
    draw: MonoBuf<'a>,
    style: MonoTextStyle<'static, BinaryColor>,
    show_border: bool,
    history: HVec<HString<LINE_CAP>, HISTORY_CAP>,
    last_line_count: usize,
    new_lines_since_render: usize,
}

impl<'a> ConsoleBuffer<'a> {
    /// Create a new console buffer with its own framebuffer
    pub fn new() -> Self {
        let fb: &mut [u8; BUF_SIZE] = CONSOLE_FB.init([0; BUF_SIZE]);
        let mut draw = MonoBuf::new(&mut fb[..], WIDTH as u32, HEIGHT as u32);
        draw.clear(Off).ok();
        let style = MonoTextStyle::new(&FONT_9X18, On);

        Self {
            draw,
            style,
            show_border: true,
            history: HVec::new(),
            last_line_count: 0,
            new_lines_since_render: 0,
        }
    }

    /// Set whether to show a border around the console
    pub fn set_border(&mut self, on: bool) {
        self.show_border = on;
    }

    /// Clear all history
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.new_lines_since_render = 0;
    }

    /// Push a line into history with timestamp
    ///
    /// Returns the refresh strategy that should be used
    pub fn push_line(&mut self, msg: &str) -> RefreshStrategy {
        let mut line: HString<LINE_CAP> = HString::new();

        // Add timestamp
        let now = Instant::now();
        let millis = now.as_millis();
        let secs = millis / 1000;
        let ms = millis % 1000;
        let _ = core::fmt::write(&mut line, core::format_args!("[{}.{:03}] ", secs, ms));

        // Add message
        let remaining = LINE_CAP.saturating_sub(line.len());
        let take = msg.len().min(remaining);
        let _ = line.push_str(core::str::from_utf8(&msg.as_bytes()[..take]).unwrap_or(""));

        // Check if we'll overflow (need full refresh)
        let will_overflow = self.history.len() == self.history.capacity();
        if will_overflow {
            let _ = self.history.remove(0);
            self.new_lines_since_render = 0; // Reset since we need full refresh
            return RefreshStrategy::Full;
        }

        let _ = self.history.push(line);
        self.new_lines_since_render += 1;

        RefreshStrategy::Partial {
            new_lines: self.new_lines_since_render,
        }
    }

    /// Render the buffer from current history state
    ///
    /// This updates the internal framebuffer but does NOT trigger any display updates
    pub fn render(&mut self) {
        self.draw.clear(Off).ok();

        // Draw border if enabled
        if self.show_border {
            Rectangle::new(Point::new(0, 0), Size::new(WIDTH as u32, HEIGHT as u32))
                .into_styled(PrimitiveStyle::with_stroke(On, 1))
                .draw(&mut self.draw)
                .ok();
        }

        // Draw visible lines
        let total = self.history.len();
        let start = total.saturating_sub(MAX_VISIBLE_LINES);
        let visible = &self.history[start..total];

        let mut y = MARGIN;
        for line in visible {
            Text::with_baseline(
                line.as_str(),
                Point::new(MARGIN, y),
                self.style,
                Baseline::Top,
            )
            .draw(&mut self.draw)
            .ok();
            y += LINE_H;
            if (y + LINE_H + MARGIN) >= HEIGHT as i32 {
                break;
            }
        }

        self.last_line_count = visible.len();
        self.new_lines_since_render = 0;
    }

    /// Get a reference to the framebuffer
    pub fn buffer(&self) -> &[u8] {
        self.draw.buffer()
    }

    /// Get the number of lines currently visible
    pub fn visible_line_count(&self) -> usize {
        self.history.len().min(MAX_VISIBLE_LINES)
    }

    /// Get the number of new lines since last render
    pub fn new_lines_count(&self) -> usize {
        self.new_lines_since_render
    }

    /// Calculate the Y start position for partial updates
    pub fn partial_update_y_start(&self, lines_to_update: usize) -> i32 {
        let current_visible = self.visible_line_count();
        let start_line = current_visible.saturating_sub(lines_to_update);
        MARGIN + (start_line as i32 * LINE_H)
    }

    /// Calculate the height for partial updates
    pub fn partial_update_height(&self, lines_to_update: usize) -> i32 {
        lines_to_update as i32 * LINE_H
    }

    /// Extract buffer data for a specific rectangle
    pub fn extract_rect_data(&self, x: usize, y: usize, w: usize, h: usize) -> HVec<u8, 4096> {
        let mut partial_buf = HVec::new();

        if w == 0 || h == 0 {
            return partial_buf;
        }

        let bytes_per_full_row = WIDTH / 8;
        let bytes_per_partial_row = w / 8;

        // Extract bytes row by row
        for dy in 0..h {
            let actual_y = y + dy;
            if actual_y >= HEIGHT {
                break;
            }

            let full_row_start = actual_y * bytes_per_full_row;
            let partial_start_byte = full_row_start + (x / 8);

            for byte_offset in 0..bytes_per_partial_row {
                let byte_idx = partial_start_byte + byte_offset;
                if byte_idx < self.draw.buf.len() && partial_buf.len() < partial_buf.capacity() {
                    let _ = partial_buf.push(self.draw.buf[byte_idx]);
                }
            }
        }

        partial_buf
    }
}

impl<'a> Default for ConsoleBuffer<'a> {
    fn default() -> Self {
        Self::new()
    }
}
