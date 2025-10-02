//! Console UI - handles display and EPD communication.
//!
//! This module provides the UI layer that coordinates between ConsoleBuffer
//! and the EPD display hardware. It makes decisions about when and how to
//! refresh the display based on buffer state.

use embassy_time::{Duration, Timer};
use embedded_hal::digital::{ErrorType as DigitalErrorType, InputPin, OutputPin};
use embedded_hal::spi::ErrorType as SpiErrorType;
use embedded_hal_async::{digital::Wait, spi::SpiBus};

use super::buffer::{ConsoleBuffer, RefreshStrategy};
use crate::epd_driver::{DriverError, Epd800x480, Rect, WIDTH};

/// Console UI controller - manages display updates
pub struct ConsoleUI<'a> {
    buffer: ConsoleBuffer<'a>,
    visible: bool,
}

impl<'a> ConsoleUI<'a> {
    /// Create a new console UI
    pub fn new() -> Self {
        Self {
            buffer: ConsoleBuffer::new(),
            visible: false,
        }
    }

    /// Get mutable reference to the buffer for configuration
    pub fn buffer_mut(&mut self) -> &mut ConsoleBuffer<'a> {
        &mut self.buffer
    }

    /// Get reference to the buffer for reading
    pub fn buffer(&self) -> &ConsoleBuffer<'a> {
        &self.buffer
    }

    /// Set border visibility
    pub fn set_border(&mut self, on: bool) {
        self.buffer.set_border(on);
    }

    /// Clear history and reset state
    pub fn clear_history(&mut self) {
        self.buffer.clear_history();
    }

    /// Show the console
    pub async fn show<SPI, CS, DC, RST, BUSY, LED>(
        &mut self,
        epd: &mut Epd800x480<SPI, CS, DC, RST, BUSY, LED>,
    ) -> Result<(), DriverError<SPI, CS>>
    where
        SPI: SpiBus<u8> + SpiErrorType,
        CS: OutputPin + DigitalErrorType,
        DC: OutputPin + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
        RST: OutputPin + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
        BUSY: InputPin + Wait + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
        LED: OutputPin,
    {
        self.visible = true;
        self.buffer.render();
        epd.display(self.buffer.buffer()).await?;
        Timer::after(Duration::from_millis(50)).await;
        Ok(())
    }

    /// Hide the console
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Check if console is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Push a line and automatically update display if visible
    pub async fn push<SPI, CS, DC, RST, BUSY, LED>(
        &mut self,
        msg: &str,
        epd: &mut Epd800x480<SPI, CS, DC, RST, BUSY, LED>,
    ) -> Result<(), DriverError<SPI, CS>>
    where
        SPI: SpiBus<u8> + SpiErrorType,
        CS: OutputPin + DigitalErrorType,
        DC: OutputPin + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
        RST: OutputPin + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
        BUSY: InputPin + Wait + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
        LED: OutputPin,
    {
        let strategy = self.buffer.push_line(msg);

        if self.visible {
            self.refresh_display(epd, strategy).await?;
        }

        Ok(())
    }

    /// Refresh display based on strategy
    async fn refresh_display<SPI, CS, DC, RST, BUSY, LED>(
        &mut self,
        epd: &mut Epd800x480<SPI, CS, DC, RST, BUSY, LED>,
        strategy: RefreshStrategy,
    ) -> Result<(), DriverError<SPI, CS>>
    where
        SPI: SpiBus<u8> + SpiErrorType,
        CS: OutputPin + DigitalErrorType,
        DC: OutputPin + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
        RST: OutputPin + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
        BUSY: InputPin + Wait + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
        LED: OutputPin,
    {
        self.buffer.render();

        match strategy {
            RefreshStrategy::Full => {
                epd.clear().await?;
                epd.display(self.buffer.buffer()).await?;
            }
            RefreshStrategy::Partial { new_lines } => {
                self.partial_update(epd, new_lines).await?;
            }
            RefreshStrategy::None => {
                // No update needed
            }
        }

        Timer::after(Duration::from_millis(50)).await;
        Ok(())
    }

    /// Perform partial display update
    async fn partial_update<SPI, CS, DC, RST, BUSY, LED>(
        &mut self,
        epd: &mut Epd800x480<SPI, CS, DC, RST, BUSY, LED>,
        new_lines: usize,
    ) -> Result<(), DriverError<SPI, CS>>
    where
        SPI: SpiBus<u8> + SpiErrorType,
        CS: OutputPin + DigitalErrorType,
        DC: OutputPin + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
        RST: OutputPin + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
        BUSY: InputPin + Wait + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
        LED: OutputPin,
    {
        if new_lines == 0 {
            return Ok(());
        }

        let current_visible = self.buffer.visible_line_count();

        // If too many new lines, do full refresh
        if new_lines >= current_visible {
            epd.display(self.buffer.buffer()).await?;
            return Ok(());
        }

        let lines_to_update = new_lines.min(current_visible);
        let y_start = self.buffer.partial_update_y_start(lines_to_update);
        let y_height = self.buffer.partial_update_height(lines_to_update);

        let rect = Rect {
            x: 0,
            y: y_start as usize,
            w: WIDTH,
            h: y_height as usize,
        };

        let partial_buf = self
            .buffer
            .extract_rect_data(rect.x, rect.y, rect.w, rect.h);

        if !partial_buf.is_empty() {
            epd.display_partial(&partial_buf, rect).await?;
        }

        Ok(())
    }

    /// Log an info message
    pub async fn log_info<SPI, CS, DC, RST, BUSY, LED>(
        &mut self,
        msg: &str,
        epd: &mut Epd800x480<SPI, CS, DC, RST, BUSY, LED>,
    ) -> Result<(), DriverError<SPI, CS>>
    where
        SPI: SpiBus<u8> + SpiErrorType,
        CS: OutputPin + DigitalErrorType,
        DC: OutputPin + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
        RST: OutputPin + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
        BUSY: InputPin + Wait + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
        LED: OutputPin,
    {
        let mut full_msg: heapless::String<96> = heapless::String::new();
        let _ = full_msg.push_str("INFO: ");
        let _ = full_msg.push_str(msg);
        self.push(&full_msg, epd).await
    }

    /// Log a warning message
    pub async fn log_warn<SPI, CS, DC, RST, BUSY, LED>(
        &mut self,
        msg: &str,
        epd: &mut Epd800x480<SPI, CS, DC, RST, BUSY, LED>,
    ) -> Result<(), DriverError<SPI, CS>>
    where
        SPI: SpiBus<u8> + SpiErrorType,
        CS: OutputPin + DigitalErrorType,
        DC: OutputPin + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
        RST: OutputPin + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
        BUSY: InputPin + Wait + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
        LED: OutputPin,
    {
        let mut full_msg: heapless::String<96> = heapless::String::new();
        let _ = full_msg.push_str("WARN: ");
        let _ = full_msg.push_str(msg);
        self.push(&full_msg, epd).await
    }

    /// Log an error message
    pub async fn log_error<SPI, CS, DC, RST, BUSY, LED>(
        &mut self,
        msg: &str,
        epd: &mut Epd800x480<SPI, CS, DC, RST, BUSY, LED>,
    ) -> Result<(), DriverError<SPI, CS>>
    where
        SPI: SpiBus<u8> + SpiErrorType,
        CS: OutputPin + DigitalErrorType,
        DC: OutputPin + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
        RST: OutputPin + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
        BUSY: InputPin + Wait + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
        LED: OutputPin,
    {
        let mut full_msg: heapless::String<96> = heapless::String::new();
        let _ = full_msg.push_str("ERROR: ");
        let _ = full_msg.push_str(msg);
        self.push(&full_msg, epd).await
    }
}

impl<'a> Default for ConsoleUI<'a> {
    fn default() -> Self {
        Self::new()
    }
}
