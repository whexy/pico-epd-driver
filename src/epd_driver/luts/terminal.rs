use super::LutSet;

const VOLTAGE_FRAME_FAST_TEXT_HC: [u8; 7] = [0x06, 0x24, 0x24, 0x08, 0x14, 0x04, 0x10];

const LUT_VCOM_FAST_TEXT_HC: [u8; 42] = [
    0x00, 0x0F, 0x0F, 0x00, 0x00, 0x02, 0x00, 0x02, 0x00, 0x02, 0x00, 0x01, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const LUT_WW_FAST_TEXT_HC: [u8; 42] = [
    0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x01, 0x00, 0x01, 0x00, 0x01, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const LUT_BW_FAST_TEXT_HC: [u8; 42] = [
    0x10, 0x0F, 0x0F, 0x00, 0x00, 0x03, 0x84, 0x05, 0x00, 0x05, 0x00, 0x01, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const LUT_WB_FAST_TEXT_HC: [u8; 42] = [
    0x80, 0x0F, 0x0F, 0x00, 0x00, 0x01, 0x40, 0x05, 0x00, 0x05, 0x00, 0x01, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const LUT_BB_FAST_TEXT_HC: [u8; 42] = [
    0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x01, 0x00, 0x01, 0x00, 0x01, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub const LUTS: LutSet = LutSet {
    voltage_frame: VOLTAGE_FRAME_FAST_TEXT_HC,
    vcom: LUT_VCOM_FAST_TEXT_HC,
    ww: LUT_WW_FAST_TEXT_HC,
    bw: LUT_BW_FAST_TEXT_HC,
    wb: LUT_WB_FAST_TEXT_HC,
    bb: LUT_BB_FAST_TEXT_HC,
};
