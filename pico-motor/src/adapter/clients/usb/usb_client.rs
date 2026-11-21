use embassy_executor::Spawner;
use embassy_rp::{
    Peri, bind_interrupts,
    peripherals::USB,
    usb::{Driver, InterruptHandler},
};
use embassy_usb::{Builder, Config, UsbDevice};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

pub type UsbDriver = Driver<'static, USB>;
pub type Usb = UsbDevice<'static, UsbDriver>;

pub struct UsbClient {
    device: Usb,
}

impl UsbClient {
    pub fn builder(usb: Peri<'static, USB>) -> Builder<'static, UsbDriver> {
        static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
        static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
        static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

        let driver = Driver::new(usb, Irqs);

        let mut config = Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("Robot Rust");
        config.product = Some("Pico2 Robot");
        config.serial_number = Some("PICO2-001");
        config.max_packet_size_0 = 64;

        Builder::new(
            driver,
            config,
            CONFIG_DESC.init([0; 256]),
            BOS_DESC.init([0; 256]),
            &mut [],
            CONTROL_BUF.init([0; 64]),
        )
    }

    pub fn new(device: Usb) -> Self {
        Self { device }
    }

    pub fn start(self, spawner: Spawner) {
        spawner.spawn(usb_task(self.device).unwrap());
    }
}

#[embassy_executor::task]
async fn usb_task(mut usb: Usb) -> ! {
    usb.run().await
}
