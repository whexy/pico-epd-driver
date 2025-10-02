// LUTs for EPD
use super::DisplayMode;
use crate::epd_driver::Epd800x480;

use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::SpiBus;

use embedded_hal::digital::ErrorType as DigitalErrorType;
use embedded_hal::spi::ErrorType as SpiErrorType;

use crate::epd_driver::command::Command;
use crate::epd_driver::error::DriverError;

impl<SPI, CS, DC, RST, BUSY, LED> Epd800x480<SPI, CS, DC, RST, BUSY, LED>
where
    SPI: SpiBus<u8> + SpiErrorType,
    CS: OutputPin + DigitalErrorType,
    DC: OutputPin + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
    RST: OutputPin + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
    BUSY: InputPin + Wait + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
    LED: OutputPin,
{
    pub async fn set_mode(&mut self, mode: DisplayMode) -> Result<(), DriverError<SPI, CS>> {
        let luts = mode.lut_set();

        // Ensure we are in Register LUT mode
        self.bus.write_cmd(Command::PanelSetting).await?;
        self.bus.write_data(&[0x3F]).await?;

        self.bus.write_cmd(Command::Btst).await?;
        self.bus.write_data(&luts.voltage_frame).await?;

        // Load the full, official 3-phase LUTs
        self.bus.write_cmd(Command::Vcom).await?;
        self.bus.write_data(&luts.vcom).await?;
        self.bus.write_cmd(Command::LutWw).await?;
        self.bus.write_data(&luts.ww).await?;
        self.bus.write_cmd(Command::LutBw).await?;
        self.bus.write_data(&luts.bw).await?;
        self.bus.write_cmd(Command::LutWb).await?;
        self.bus.write_data(&luts.wb).await?;
        self.bus.write_cmd(Command::LutBb).await?;
        self.bus.write_data(&luts.bb).await?;

        Ok(())
    }
}
