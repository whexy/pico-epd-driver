mod bus;
mod command;
mod driver;
mod error;

pub const WIDTH: usize = 800;
pub const HEIGHT: usize = 480;
pub const BUF_SIZE: usize = (WIDTH * HEIGHT) / 8;

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

pub use bus::EpdBus;
pub use driver::Epd800x480;
pub use error::DriverError;
