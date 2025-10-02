use super::bus::EpdBusError;
use embedded_hal::digital::ErrorType as DigitalErrorType;
use embedded_hal::spi::ErrorType as SpiErrorType;

#[derive(Debug)]
pub enum EpdDriverError<SpiE, GpioE> {
    Bus(EpdBusError<SpiE, GpioE>),
    BadBufferLen { expected: usize, got: usize },
}

impl<SpiE, GpioE> From<EpdBusError<SpiE, GpioE>> for EpdDriverError<SpiE, GpioE> {
    fn from(e: EpdBusError<SpiE, GpioE>) -> Self {
        Self::Bus(e)
    }
}

pub type DriverError<SPI, CS> =
    EpdDriverError<<SPI as SpiErrorType>::Error, <CS as DigitalErrorType>::Error>;
