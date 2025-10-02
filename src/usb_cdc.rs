use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State as CdcState};
use embassy_usb::{Builder, Config, UsbDevice};
use static_cell::StaticCell;

// Bind RP USB interrupt to Embassy's handler (type-level).
bind_interrupts!(pub struct Irqs {
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<USB>;
});

pub struct UsbParts {
    pub device: UsbDevice<'static, Driver<'static, USB>>,
    pub cdc: CdcAcmClass<'static, Driver<'static, USB>>,
}

// Descriptor arenas required by embassy-usb.
static DEVICE_DESC: StaticCell<[u8; 256]> = StaticCell::new();
static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
static CTRL_BUF: StaticCell<[u8; 256]> = StaticCell::new();
static CDC_STATE: StaticCell<CdcState> = StaticCell::new();

/// Initialize USB CDC with a prebuilt low-level Driver.
/// (This avoids lifetime/type gymnastics around `Peri<'d, USB>`.)
pub fn init(driver: Driver<'static, USB>) -> UsbParts {
    let mut cfg = Config::new(0xCAFE, 0xE411);
    cfg.manufacturer = Some("Wenxuan Labs");
    cfg.product = Some("EPD Uploader");
    cfg.serial_number = Some("EPD-1999");
    cfg.max_power = 100;
    cfg.max_packet_size_0 = 64;

    let dev_desc = DEVICE_DESC.init([0; 256]);
    let cfg_desc = CONFIG_DESC.init([0; 256]);
    let bos_desc = BOS_DESC.init([0; 256]);
    let ctrl_buf = CTRL_BUF.init([0; 256]);

    let mut builder = Builder::new(driver, cfg, dev_desc, cfg_desc, bos_desc, ctrl_buf);

    // CDC class
    let cdc_state = CDC_STATE.init(CdcState::new());
    let cdc = CdcAcmClass::new(&mut builder, cdc_state, 64);

    let device = builder.build();
    UsbParts { device, cdc }
}
