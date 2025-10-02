// LUTs for EPD

pub mod fast;
pub mod mode;
pub mod official;

#[derive(Debug)]
pub struct LutSet {
    pub voltage_frame: [u8; 7],
    pub vcom: [u8; 42],
    pub ww: [u8; 42],
    pub bw: [u8; 42],
    pub wb: [u8; 42],
    pub bb: [u8; 42],
}

/// Enum to select a display refresh mode.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DisplayMode {
    /// Full, multi-phase refresh for best image quality.
    Official,
    /// Fast refresh mode.
    Fast,
}

impl DisplayMode {
    pub fn lut_set(&self) -> LutSet {
        match self {
            DisplayMode::Official => official::LUTS,
            DisplayMode::Fast => fast::LUTS,
        }
    }
}
