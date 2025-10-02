#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Command {
    PanelSetting = 0x00,
    PowerSetting = 0x01,
    PowerOff = 0x02,
    PowerOn = 0x04,
    Btst = 0x06, // Booster Soft Start
    DeepSleep = 0x07,
    DataStartTransmission1 = 0x10,
    DisplayRefresh = 0x12,
    DataStartTransmission2 = 0x13,
    DualSPI = 0x15,
    Vcom = 0x20,
    LutWw = 0x21,
    LutBw = 0x22,
    LutWb = 0x23,
    LutBb = 0x24,
    LutOpt = 0x2A,
    Pll = 0x30,
    Tsc = 0x40,
    Tse = 0x41,
    VcomAndDataInterval = 0x50,
    Evs = 0x52,
    TconSetting = 0x60,
    TRes = 0x61,
    ResolutionSetting = 0x65,
    GetStatus = 0x71,
    VcomDc = 0x82,
    PartialWindow = 0x90,
    PartialIn = 0x91,
    PartialOut = 0x92,
}

impl From<Command> for u8 {
    #[inline]
    fn from(c: Command) -> Self {
        c as u8
    }
}
