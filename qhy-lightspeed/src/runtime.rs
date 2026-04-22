use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use astrotools::Lightspeed;
use astrotools::properties::UpdatePropertyRequest;

use crate::{ExposureValue, QhyLightspeed};

pub enum Command {
    Update(UpdatePropertyRequest),
    Expose(ExposureValue),
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

fn publish_frame(frame: crate::Frame, topic: &str, state_tx: &mpsc::SyncSender<OutboundMessage>) {
    if let Ok(payload) = serde_json::to_string(&frame) {
        let _ = state_tx.try_send(OutboundMessage {
            topic: format!("{}/frame", topic),
            payload,
        });
    }
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

        if let Some(frame) = device.try_collect_frame() {
            publish_frame(frame, &topic, &state_tx);
        }

        loop {
            match cmd_rx.try_recv() {
                Ok(Command::Shutdown) => return,
                Ok(Command::Expose(val)) => {
                    if let Ok(Some(frame)) = device.start_exposure(val) {
                        publish_frame(frame, &topic, &state_tx);
                    }
                }
                Ok(Command::Update(req)) => {
                    let _ = device.update_property(&req.prop_name, req.value);
                }
                Err(_) => break,
            }
        }

        if let Some(remaining) = interval.checked_sub(tick.elapsed()) {
            thread::sleep(remaining);
        }
    }
}
