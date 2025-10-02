pub mod buffer;
pub mod ui;

use embedded_hal::digital::{ErrorType as DigitalErrorType, InputPin, OutputPin};
use embedded_hal::spi::ErrorType as SpiErrorType;
use embedded_hal_async::{digital::Wait, spi::SpiBus};

use crate::epd_driver::{DriverError, Epd800x480};
use ui::ConsoleUI;

/// Simplified EPD console with automatic refresh decisions.
///
/// This is a backwards-compatible wrapper around ConsoleUI.
pub struct EpdConsole<'a, SPI, CS, DC, RST, BUSY, LED> {
    ui: ConsoleUI<'a>,
    epd: &'a mut Epd800x480<SPI, CS, DC, RST, BUSY, LED>,
}

impl<'a, SPI, CS, DC, RST, BUSY, LED> EpdConsole<'a, SPI, CS, DC, RST, BUSY, LED>
where
    SPI: SpiBus<u8> + SpiErrorType,
    CS: OutputPin + DigitalErrorType,
    DC: OutputPin + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
    RST: OutputPin + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
    BUSY: InputPin + Wait + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
    LED: OutputPin,
{
    /// Create a console with its own framebuffer.
    pub fn new(epd: &'a mut Epd800x480<SPI, CS, DC, RST, BUSY, LED>) -> Self {
        Self {
            ui: ConsoleUI::new(),
            epd,
        }
    }

    /// Push a line into history and automatically refresh the display.
    pub async fn push(&mut self, s: &str) -> Result<(), DriverError<SPI, CS>> {
        self.ui.push(s, self.epd).await
    }

    /// Show the console (makes it visible and refreshes)
    pub async fn show(&mut self) -> Result<(), DriverError<SPI, CS>> {
        self.ui.show(self.epd).await
    }

    /// Hide the console
    pub fn hide(&mut self) {
        self.ui.hide();
    }
}
