use embassy_time::{Duration, Timer};
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::SpiBus;

// Error Types
use embedded_hal::digital::ErrorType as DigitalErrorType;
use embedded_hal::spi::ErrorType as SpiErrorType;

use super::bus::EpdBus;
use super::command::Command;
use super::error::{DriverError, EpdDriverError};
use super::{BUF_SIZE, HEIGHT, Rect, WIDTH};

pub struct Epd800x480<SPI, CS, DC, RST, BUSY, LED> {
    pub bus: EpdBus<SPI, CS, DC, RST, BUSY>,
    led: LED,
}

impl<SPI, CS, DC, RST, BUSY, LED> Epd800x480<SPI, CS, DC, RST, BUSY, LED>
where
    SPI: SpiBus<u8> + SpiErrorType,
    CS: OutputPin + DigitalErrorType,
    DC: OutputPin + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
    RST: OutputPin + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
    BUSY: InputPin + Wait + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
    LED: OutputPin,
{
    pub fn new(bus: EpdBus<SPI, CS, DC, RST, BUSY>, led: LED) -> Self {
        Self { bus, led }
    }

    pub async fn hw_reset(&mut self) -> Result<(), DriverError<SPI, CS>> {
        self.bus
            .reset(20, 2, 20)
            .await
            .map_err(EpdDriverError::from)
    }

    pub async fn wait_ready(&mut self) -> Result<(), DriverError<SPI, CS>> {
        let _ = self.led.set_high();
        self.bus.write_cmd(Command::GetStatus).await?;
        Timer::after(Duration::from_millis(20)).await;
        match self.bus.wait().await {
            Ok(t) => {
                let _ = self.led.set_low();
                Ok(t)
            }
            Err(e) => Err(EpdDriverError::from(e)),
        }
    }

    pub async fn init(&mut self) -> Result<(), DriverError<SPI, CS>> {
        self.hw_reset().await?;

        // Power Setting (PWR)
        self.bus.write_cmd(Command::PowerSetting).await?;
        self.bus.write_data(&[0x17, 0x17, 0x3f, 0x3f, 0x11]).await?;

        // VCOM and data interval setting
        self.bus.write_cmd(Command::VcomDc).await?;
        self.bus.write_data(&[0x24]).await?;

        // Booster Soft Start
        self.bus.write_cmd(Command::Btst).await?;
        self.bus.write_data(&[0x27, 0x27, 0x2F, 0x17]).await?;

        // PLL Control
        self.bus.write_cmd(Command::Pll).await?;
        self.bus.write_data(&[0x06]).await?; // 50Hz refresh rate

        // Power On
        self.bus.write_cmd(Command::PowerOn).await?;
        self.wait_ready().await?;
        self.bus.write_cmd(Command::PanelSetting).await?;
        self.bus.write_data(&[0x1F]).await?;
        self.bus.write_cmd(Command::TRes).await?;
        self.bus
            .write_data(&[
                hb(WIDTH as u16),
                lb(WIDTH as u16),
                hb(HEIGHT as u16),
                lb(HEIGHT as u16),
            ])
            .await?;
        self.bus.write_cmd(Command::DualSPI).await?;
        self.bus.write_data(&[0x00]).await?;
        self.bus.write_cmd(Command::VcomAndDataInterval).await?;
        self.bus.write_data(&[0x10, 0x07]).await?;
        self.bus.write_cmd(Command::TconSetting).await?;
        self.bus.write_data(&[0x22]).await?;
        Ok(())
    }

    pub async fn display(&mut self, buf: &[u8]) -> Result<(), DriverError<SPI, CS>> {
        self.wait_ready().await?;
        if buf.len() != BUF_SIZE {
            return Err(EpdDriverError::BadBufferLen {
                expected: BUF_SIZE,
                got: buf.len(),
            });
        }
        self.bus.write_cmd(Command::DataStartTransmission1).await?;
        self.send_zeros(BUF_SIZE).await?;
        self.bus.write_cmd(Command::DataStartTransmission2).await?;
        self.bus.write_data(buf).await?;
        self.refresh().await?;
        Ok(())
    }

    pub async fn display_partial(
        &mut self,
        buf: &[u8],
        r: Rect,
    ) -> Result<(), DriverError<SPI, CS>> {
        self.wait_ready().await?;
        let x_start = r.x;
        let x_end = r.x + r.w - 1;
        let y_start = r.y;
        let y_end = r.y + r.h - 1;
        self.bus.write_cmd(Command::PartialIn).await?;
        self.bus.write_cmd(Command::PartialWindow).await?;
        self.bus
            .write_data(&[
                (x_start >> 8) as u8,
                (x_start & 0xFF) as u8,
                (x_end >> 8) as u8,
                (x_end & 0xFF) as u8,
                (y_start >> 8) as u8,
                (y_start & 0xFF) as u8,
                (y_end >> 8) as u8,
                (y_end & 0xFF) as u8,
                0x01,
            ])
            .await?;
        self.bus.write_cmd(Command::DataStartTransmission2).await?;
        self.bus.write_data(buf).await?;
        self.refresh().await?;
        self.bus.write_cmd(Command::PartialOut).await?;
        Ok(())
    }

    pub async fn clear(&mut self) -> Result<(), DriverError<SPI, CS>> {
        self.wait_ready().await?;
        self.bus.write_cmd(Command::DataStartTransmission1).await?;
        self.send_zeros(BUF_SIZE).await?;
        self.bus.write_cmd(Command::DataStartTransmission2).await?;
        self.send_zeros(BUF_SIZE).await?;
        self.refresh().await?;
        Ok(())
    }

    pub async fn sleep(&mut self) -> Result<(), DriverError<SPI, CS>> {
        self.bus.write_cmd(Command::PowerOff).await?;
        self.wait_ready().await?;
        self.bus.write_cmd(Command::DeepSleep).await?;
        self.bus.write_data(&[0xA5]).await?;
        Ok(())
    }

    pub async fn flash_led(&mut self) {
        Timer::after(Duration::from_millis(500)).await;
        let _ = self.led.set_high();
        Timer::after(Duration::from_millis(500)).await;
        let _ = self.led.set_low();
    }

    async fn refresh(&mut self) -> Result<(), DriverError<SPI, CS>> {
        self.bus.write_cmd(Command::DisplayRefresh).await?;
        self.wait_ready().await
    }
    async fn send_zeros(&mut self, total: usize) -> Result<(), DriverError<SPI, CS>> {
        let block = [0u8; HEIGHT * WIDTH / 8]; // maximum size is height * width / 8
        self.bus.write_data(&block[..total]).await?;
        Ok(())
    }
}

#[inline(always)]
fn hb(x: u16) -> u8 {
    (x >> 8) as u8
}
#[inline(always)]
fn lb(x: u16) -> u8 {
    (x & 0xFF) as u8
}
