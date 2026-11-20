# Compact Embassy firmware example

この例は、`embassy-rp 0.10` と `embassy-usb 0.6` を使って、次の二方向を扱う最小構成です。

- Jetson/Linux → USB CDC-ACM → `FWD` / `BACK` / `STOP` → モーターGPIO
- PIRセンサーGPIO → USB CDC-ACM → Jetson/Linux

USBからの指示は既存の `robot_protocol` 形式（例: `42 FWD\n`）で受信します。センサー通知は例として `PIR 1\n` / `PIR 0\n` を送ります。この通知形式は現在の `robot_protocol` にはないため、Linux側で個別に読むか、共有プロトコルへ追加してください。

以下は `src/main.rs` に置くコード例です。GPIO6はPIR入力の仮のピン番号です。センサーの配線・出力極性に合わせてピン番号と `Pull` を調整してください。

現在の `Cargo.toml` では `cortex-m-rt` が直接依存になっていないため、起動属性の `entry` だけ既存 `rp235x-hal` の `arch_entry` を参照しています。ペリフェラル初期化・GPIO・USB・タイマーは `embassy-rp` / `embassy-usb` で扱います。

```rust
#![no_std]
#![no_main]

mod domain;

use defmt::unwrap;
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    gpio::{Input, Level, Output, Pull},
    peripherals::USB,
    usb::{Driver, InterruptHandler},
};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    signal::Signal,
};
use embassy_time::Timer;
use embassy_usb::{
    Builder,
    class::cdc_acm::{CdcAcmClass, Receiver, Sender, State},
};
use panic_probe as _;
use static_cell::StaticCell;

use robot_protocol::{self as protocol, Command};
use crate::domain::motor::Motion;

const STEP_INTERVAL_US: u64 = 5_000;
const STEPS: [u8; 8] = [1, 3, 2, 6, 4, 12, 8, 9];

// USB RXからモータータスクへ最新の目標状態を渡す。
static MOTION: Signal<CriticalSectionRawMutex, Motion> = Signal::new();

// PIRの最新状態をUSB TXへ渡す。
static PIR_STATE: Signal<CriticalSectionRawMutex, bool> = Signal::new();

// USB descriptorとCDCの状態は静的寿命が必要。
static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();
static CDC_STATE: StaticCell<State<'static>> = StaticCell::new();

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

type UsbDriver = Driver<'static, USB>;
type UsbDevice = embassy_usb::UsbDevice<'static, UsbDriver>;
type CdcRx = Receiver<'static, UsbDriver>;
type CdcTx = Sender<'static, UsbDriver>;

#[embassy_executor::task]
async fn usb_device_task(mut device: UsbDevice) -> ! {
    device.run().await
}

// 1行のコマンドを解釈し、モータータスクへ渡す。
fn handle_command_line(line: &[u8]) {
    let Ok(line) = core::str::from_utf8(line) else {
        return;
    };
    let Ok(request) = protocol::decode_request(line) else {
        return;
    };

    let motion = match request.command {
        Command::Forward => Some(Motion::Forward),
        Command::Backward => Some(Motion::Backward),
        Command::Stop => Some(Motion::Stopped),
        Command::Status | Command::Ping => None,
    };

    if let Some(motion) = motion {
        MOTION.signal(motion);
    }
}

// USB OUT endpointを読むタスク。パケットを改行区切りの行に組み立てる。
#[embassy_executor::task]
async fn usb_rx_task(mut rx: CdcRx) {
    let mut packet = [0u8; 64];
    let mut line = [0u8; 64];
    let mut line_len = 0usize;
    let mut discard_line = false;

    loop {
        rx.wait_connection().await;

        let count = match rx.read_packet(&mut packet).await {
            Ok(count) => count,
            Err(_) => {
                line_len = 0;
                discard_line = false;
                continue;
            }
        };

        for &byte in &packet[..count] {
            if byte == b'\n' {
                if !discard_line {
                    handle_command_line(&line[..line_len]);
                }
                line_len = 0;
                discard_line = false;
            } else if byte == b'\r' {
                // CRは無視する。
            } else if !discard_line {
                if line_len < line.len() {
                    line[line_len] = byte;
                    line_len += 1;
                } else {
                    // 長すぎる行は次の改行まで捨てる。
                    line_len = 0;
                    discard_line = true;
                }
            }
        }
    }
}

// PIR入力の状態変化を待つ。GPIO6は例。
#[embassy_executor::task]
async fn pir_task(mut pir: Input<'static>) {
    let mut previous = pir.is_high();
    PIR_STATE.signal(previous);

    loop {
        pir.wait_for_any_edge().await;
        let detected = pir.is_high();

        if detected != previous {
            previous = detected;
            PIR_STATE.signal(detected);
        }
    }
}

// USB IN endpointからPIR状態をホストへ送る。
#[embassy_executor::task]
async fn pir_usb_tx_task(mut tx: CdcTx) {
    let mut retry = None;

    loop {
        tx.wait_connection().await;

        let detected = match retry.take() {
            Some(state) => state,
            None => PIR_STATE.wait().await,
        };

        let message: &[u8] = if detected {
            b"PIR 1\n"
        } else {
            b"PIR 0\n"
        };

        if tx.write_packet(message).await.is_err() {
            retry = Some(detected);
        }
    }
}

fn set_pattern(pins: &mut [Output<'static>; 4], pattern: u8) {
    for (bit, pin) in pins.iter_mut().enumerate() {
        if pattern & (1u8 << bit) != 0 {
            pin.set_high();
        } else {
            pin.set_low();
        }
    }
}

// モーターGPIOを所有し、目標状態に応じてコイルを駆動する。
#[embassy_executor::task]
async fn motor_task(mut pins: [Output<'static>; 4]) {
    let mut motion = Motion::Stopped;
    let mut phase = 0usize;

    loop {
        if let Some(next) = MOTION.try_take() {
            motion = next;
        }

        match motion {
            Motion::Stopped => {
                for pin in &mut pins {
                    pin.set_low();
                }
            }
            Motion::Forward => {
                phase = (phase + 1) % STEPS.len();
                set_pattern(&mut pins, STEPS[phase]);
            }
            Motion::Backward => {
                phase = (phase + STEPS.len() - 1) % STEPS.len();
                set_pattern(&mut pins, STEPS[phase]);
            }
        }

        Timer::after_micros(STEP_INTERVAL_US).await;
    }
}

#[embassy_executor::main(
    entry = "rp235x_hal::arch_entry"
)]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // ULN2003 IN1..IN4 -> GP2..GP5
    let motor_pins = [
        Output::new(p.PIN_2, Level::Low),
        Output::new(p.PIN_3, Level::Low),
        Output::new(p.PIN_4, Level::Low),
        Output::new(p.PIN_5, Level::Low),
    ];
    spawner.spawn(unwrap!(motor_task(motor_pins)));

    // PIR OUT -> GP6（配線とセンサーに合わせて変更）
    let pir = Input::new(p.PIN_6, Pull::Down);
    spawner.spawn(unwrap!(pir_task(pir)));

    // USB CDC-ACMを初期化する。
    let driver = Driver::new(p.USB, Irqs);
    let mut config = embassy_usb::Config::new(0x16c0, 0x27dd);
    config.product = Some("Pico2 Motor");

    let mut builder = Builder::new(
        driver,
        config,
        CONFIG_DESC.init([0; 256]),
        BOS_DESC.init([0; 256]),
        &mut [],
        CONTROL_BUF.init([0; 64]),
    );

    let cdc = CdcAcmClass::new(
        &mut builder,
        CDC_STATE.init(State::new()),
        64,
    );
    let device = builder.build();

    // USBデバイス実行、受信、送信を別タスクに分ける。
    let (tx, rx) = cdc.split();
    spawner.spawn(unwrap!(usb_device_task(device)));
    spawner.spawn(unwrap!(usb_rx_task(rx)));
    spawner.spawn(unwrap!(pir_usb_tx_task(tx)));

    // mainタスクはここでは何もせず、executorに他タスクを実行させる。
    core::future::pending::<()>().await;
}
```

Linux/Jetsonからは、認識された `/dev/ttyACM*` へ `42 FWD\n`、`43 BACK\n`、`44 STOP\n` を送ります。PIR通知は同じCDCポートから `PIR 1\n` または `PIR 0\n` として届きます。

`CdcAcmClass::split()` でUSBの送信側と受信側を分けるため、コマンド受信とセンサー通知を別タスクから扱えます。`MOTION` と `PIR_STATE` は最新状態を保持する `Signal` です。短時間のセンサー変化を一件ずつ確実に記録したい場合は、センサー側を `Channel` に変更してください。

このコンパクト例には、コマンド応答、既存の1.5秒コマンドタイムアウト、ハードウェアwatchdog、PIR検出時の安全停止、センサーのデバウンス処理は含めていません。
