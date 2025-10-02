use embassy_time::{Duration, Timer};
use embedded_hal::digital::{ErrorType as DigitalErrorType, InputPin, OutputPin};
use embedded_hal::spi::ErrorType as SpiErrorType;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::SpiBus;

#[derive(Debug)]
pub enum EpdBusError<SpiE, GpioE> {
    Spi(SpiE),
    Gpio(GpioE),
}

pub struct EpdBus<SPI, CS, DC, RST, BUSY> {
    spi: SPI,
    cs: CS,
    dc: DC,
    rst: RST,
    busy: BUSY,
}

impl<SPI, CS, DC, RST, BUSY> EpdBus<SPI, CS, DC, RST, BUSY> {
    pub fn new(spi: SPI, cs: CS, dc: DC, rst: RST, busy: BUSY) -> Self {
        Self {
            spi,
            cs,
            dc,
            rst,
            busy,
        }
    }
}

impl<SPI, CS, DC, RST, BUSY> EpdBus<SPI, CS, DC, RST, BUSY>
where
    SPI: SpiBus<u8> + SpiErrorType,
    CS: OutputPin + DigitalErrorType,
    DC: OutputPin + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
    RST: OutputPin + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
    BUSY: InputPin + Wait + DigitalErrorType<Error = <CS as DigitalErrorType>::Error>,
{
    pub async fn reset(
        &mut self,
        t1_high_ms: u64, // e.g. 200
        t_low_ms: u64,   // e.g. 2
        t2_high_ms: u64, // e.g. 200
    ) -> Result<(), EpdBusError<<SPI as SpiErrorType>::Error, <CS as DigitalErrorType>::Error>>
    {
        self.rst.set_high().map_err(EpdBusError::Gpio)?;
        Timer::after(Duration::from_millis(t1_high_ms)).await;
        self.rst.set_low().map_err(EpdBusError::Gpio)?;
        Timer::after(Duration::from_millis(t_low_ms)).await;
        self.rst.set_high().map_err(EpdBusError::Gpio)?;
        Timer::after(Duration::from_millis(t2_high_ms)).await;
        Ok(())
    }

    pub async fn write_cmd<C: Into<u8>>(
        &mut self,
        cmd: C,
    ) -> Result<(), EpdBusError<<SPI as SpiErrorType>::Error, <CS as DigitalErrorType>::Error>>
    {
        let byte: u8 = cmd.into();
        self.dc.set_low().map_err(EpdBusError::Gpio)?;
        self.cs.set_low().map_err(EpdBusError::Gpio)?;
        let r = self.spi.write(&[byte]).await;
        // always release CS even if SPI fails
        let cs_res = self.cs.set_high().map_err(EpdBusError::Gpio);
        r.map_err(EpdBusError::Spi)?;
        cs_res?;
        Ok(())
    }

    pub async fn write_data(
        &mut self,
        data: &[u8],
    ) -> Result<(), EpdBusError<<SPI as SpiErrorType>::Error, <CS as DigitalErrorType>::Error>>
    {
        if data.is_empty() {
            return Ok(());
        }
        self.dc.set_high().map_err(EpdBusError::Gpio)?;
        self.cs.set_low().map_err(EpdBusError::Gpio)?;
        let r = self.spi.write(data).await;
        let cs_res = self.cs.set_high().map_err(EpdBusError::Gpio);
        r.map_err(EpdBusError::Spi)?;
        cs_res?;
        Ok(())
    }

    pub async fn wait(
        &mut self,
    ) -> Result<(), EpdBusError<<SPI as SpiErrorType>::Error, <CS as DigitalErrorType>::Error>>
    {
        self.busy.wait_for_high().await.map_err(EpdBusError::Gpio)?;
        Ok(())
    }
}
