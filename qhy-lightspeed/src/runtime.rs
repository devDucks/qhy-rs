use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use astrotools::Lightspeed;

use crate::QhyLightspeed;

pub enum Command {
    SetGain(f64),
    SetOffset(f64),
    SetExposure(f64),
    SetBin(u32),
    Shutdown,
}

pub struct OutboundMessage {
    pub topic: String,
    pub payload: String,
}

pub struct DeviceHandle {
    pub camera_id: String,
    pub cmd_tx: mpsc::SyncSender<Command>,
    pub thread: JoinHandle<()>,
}

pub fn spawn_devices(
    devices: Vec<QhyLightspeed>,
    interval: Duration,
    state_tx: mpsc::SyncSender<OutboundMessage>,
) -> Vec<DeviceHandle> {
    devices
        .into_iter()
        .map(|device| {
            let camera_id = device.camera_id().to_string();
            let (cmd_tx, cmd_rx) = mpsc::sync_channel(32);
            let state_tx = state_tx.clone();
            let topic = format!("devices/{}", camera_id);

            let thread = thread::spawn(move || {
                run_device(device, cmd_rx, state_tx, topic, interval);
            });

            DeviceHandle {
                camera_id,
                cmd_tx,
                thread,
            }
        })
        .collect()
}

fn run_device(
    mut device: QhyLightspeed,
    cmd_rx: mpsc::Receiver<Command>,
    state_tx: mpsc::SyncSender<OutboundMessage>,
    topic: String,
    interval: Duration,
) {
    loop {
        let tick = Instant::now();

        device.sync_state();

        if let Ok(payload) = serde_json::to_string(&device) {
            let _ = state_tx.try_send(OutboundMessage {
                topic: topic.clone(),
                payload,
            });
        }

        loop {
            match cmd_rx.try_recv() {
                Ok(Command::Shutdown) => return,
                Ok(cmd) => handle_command(&mut device, cmd),
                Err(_) => break,
            }
        }

        if let Some(remaining) = interval.checked_sub(tick.elapsed()) {
            thread::sleep(remaining);
        }
    }
}

fn handle_command(device: &mut QhyLightspeed, cmd: Command) {
    match cmd {
        Command::SetGain(v) => {
            let _ = device.set_gain(v);
        }
        Command::SetOffset(v) => {
            let _ = device.set_offset(v);
        }
        Command::SetExposure(v) => {
            let _ = device.set_exposure(v);
        }
        Command::SetBin(v) => {
            let _ = device.set_bin(v);
        }
        Command::Shutdown => {}
    }
}
