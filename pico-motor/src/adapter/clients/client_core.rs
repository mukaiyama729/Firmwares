use core::sync::atomic::{AtomicU8, Ordering};

use embassy_executor::SpawnError;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ClientStatus {
    Uninitialized = 0,
    Starting = 1,
    Running = 2,
}

#[derive(Debug)]
pub enum ClientStartError {
    AlreadyStarted,
    Spawn(SpawnError),
}

#[derive(Debug)]
pub enum ClientSendError {
    NotRunning,
}

/// C: コマンドの型
/// N: Channelのバッファサイズ
pub struct ClientCore<C, const N: usize> {
    status: AtomicU8,
    commands: Channel<CriticalSectionRawMutex, C, N>,
}

impl<C, const N: usize> ClientCore<C, N> {
    pub const fn new() -> Self {
        Self {
            status: AtomicU8::new(ClientStatus::Uninitialized as u8),
            commands: Channel::new(),
        }
    }

    pub fn begin_start(&self) -> Result<(), ClientStartError> {
        self.status
            .compare_exchange(
                ClientStatus::Uninitialized as u8,
                ClientStatus::Starting as u8,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .map(|_| ())
            .map_err(|_| ClientStartError::AlreadyStarted)
    }

    pub fn rollback_start(&self) {
        self.status
            .store(ClientStatus::Uninitialized as u8, Ordering::Release);
    }

    pub fn mark_running(&self) {
        self.status
            .store(ClientStatus::Running as u8, Ordering::Release);
    }

    pub fn status(&self) -> ClientStatus {
        match self.status.load(Ordering::Acquire) {
            0 => ClientStatus::Uninitialized,
            1 => ClientStatus::Starting,
            2 => ClientStatus::Running,
            _ => unreachable!("invalid client status"),
        }
    }

    pub fn is_started(&self) -> bool {
        self.status() != ClientStatus::Uninitialized
    }

    pub fn is_running(&self) -> bool {
        self.status() == ClientStatus::Running
    }

    pub async fn send(&self, command: C) -> Result<(), ClientSendError> {
        if !self.is_running() {
            return Err(ClientSendError::NotRunning);
        }

        self.commands.send(command).await;
        Ok(())
    }

    pub async fn receive(&self) -> C {
        self.commands.receive().await
    }
}
