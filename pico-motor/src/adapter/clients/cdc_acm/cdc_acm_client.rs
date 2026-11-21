use embassy_executor::Spawner;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

use embassy_usb::{
    Builder,
    class::cdc_acm::{CdcAcmClass, Receiver, Sender, State},
};

use heapless::{String, Vec};
use static_cell::StaticCell;

use crate::adapter::clients::usb::usb_client::UsbDriver;

pub type CdcSender = Sender<'static, UsbDriver>;
pub type CdcReceiver = Receiver<'static, UsbDriver>;

const USB_PACKET_SIZE: usize = 64;
const CHANNEL_SIZE: usize = 8;

pub type RequestFrame = Vec<u8, 128>;

// 送信する1メッセージの最大サイズ
pub type ResponseFrame = String<63>;

// rx用Channel
type RequestQueue = Channel<CriticalSectionRawMutex, RequestFrame, CHANNEL_SIZE>;

// tx用Channel
type ResponseQueue = Channel<CriticalSectionRawMutex, ResponseFrame, CHANNEL_SIZE>;

pub struct CdcAcmClient {
    requests: RequestQueue,
    responses: ResponseQueue,
}

impl CdcAcmClient {
    pub const fn new() -> Self {
        Self {
            requests: Channel::new(),
            responses: Channel::new(),
        }
    }

    pub fn register(
        &'static self,
        builder: &mut Builder<'static, UsbDriver>,
    ) -> (CdcSender, CdcReceiver) {
        static STATE: StaticCell<State> = StaticCell::new();

        let state = STATE.init(State::new());

        let class = CdcAcmClass::new(builder, state, USB_PACKET_SIZE as u16);

        class.split()
    }

    pub fn start(&'static self, spawner: Spawner, sender: CdcSender, receiver: CdcReceiver) {
        spawner.spawn(tx_worker(self, sender).unwrap());

        spawner.spawn(rx_worker(self, receiver).unwrap());
    }

    // 外部公開API

    // Presenterが使用
    pub async fn send(&self, response: ResponseFrame) {
        self.responses.send(response).await;
    }

    // Presenterが使用（待機しない）
    pub fn try_send(&self, response: ResponseFrame) -> Result<(), ()> {
        self.responses.try_send(response).map_err(|_| ())
    }

    // Controllerが使用
    pub async fn receive(&self) -> RequestFrame {
        self.requests.receive().await
    }

    // Worker専用API

    // tx_workerが使用
    async fn next_response(&self) -> ResponseFrame {
        self.responses.receive().await
    }

    // rx_workerが使用
    async fn push_request(&self, request: RequestFrame) {
        self.requests.send(request).await;
    }
}

// Pico2 -> Jetsonなどへ送信する用途
// controllerが&'static CdcAcmClientを用いてself.send, self.recieveをする
// worker内の&'static
// CdcAcmClientのnext_response.awaitやpush_request.awaitが実行されusbを通して通信がなされる
#[embassy_executor::task]
async fn tx_worker(client: &'static CdcAcmClient, mut sender: CdcSender) {
    loop {
        // USB接続を待機
        sender.wait_connection().await;

        loop {
            // TX Channelからデータを取得
            let response = client.next_response().await;

            // JetsonへUSB通信
            if sender.write_packet(response.as_bytes()).await.is_err() {
                break;
            }
        }
    }
}

#[embassy_executor::task]
async fn rx_worker(client: &'static CdcAcmClient, mut receiver: CdcReceiver) {
    let mut buffer = [0u8; USB_PACKET_SIZE];

    loop {
        // USB接続を待機
        receiver.wait_connection().await;

        let mut frame = RequestFrame::new();
        let mut overflow = false;

        loop {
            let size = match receiver.read_packet(&mut buffer).await {
                Ok(size) => size,
                Err(_) => break,
            };

            for &byte in &buffer[..size] {
                // 改行がメッセージの区切り
                if byte == b'\n' {
                    if !overflow && !frame.is_empty() {
                        let request = core::mem::take(&mut frame);

                        client.push_request(request).await;
                    }

                    frame.clear();
                    overflow = false;

                    continue;
                }

                if overflow {
                    continue;
                }

                if frame.push(byte).is_err() {
                    frame.clear();
                    overflow = true;
                }
            }
        }
    }
}
