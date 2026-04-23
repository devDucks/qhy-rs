use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Duration;

use clap::Parser;
use rumqttc::{Client, Event, MqttOptions, Packet, QoS};

use qhy_lightspeed::QhyLightspeed;
use qhy_lightspeed::runtime::{Command, spawn_devices};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    #[arg(long, default_value_t = 1883)]
    port: u16,
}

fn dispatch(topic: &str, payload: &[u8], cmd_map: &HashMap<String, mpsc::SyncSender<Command>>) {
    // topic: devices/{camera_id}/{action}
    let mut parts = topic.splitn(3, '/');
    let (Some("devices"), Some(camera_id), Some(action)) =
        (parts.next(), parts.next(), parts.next())
    else {
        return;
    };

    let Some(tx) = cmd_map.get(camera_id) else {
        log::info!("MQTT: unknown camera {camera_id}");
        return;
    };

    let cmd = match action {
        "expose" => match serde_json::from_slice(payload) {
            Ok(val) => Command::Expose(val),
            Err(e) => {
                log::error!("MQTT expose parse error: {e}");
                return;
            }
        },
        "set" => match serde_json::from_slice(payload) {
            Ok(req) => Command::Update(req),
            Err(e) => {
                log::error!("MQTT set parse error: {e}");
                return;
            }
        },
        _ => return,
    };

    if tx.try_send(cmd).is_err() {
        log::error!("MQTT: command queue full for {camera_id}");
    }
}

fn main() {
    let args = Args::parse();
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let mut ctx = libqhy::init_sdk().expect("QHY SDK init failed");
    log::info!("SDK {}", ctx.sdk_version());

    let cameras = std::mem::take(&mut ctx.cameras);
    let devices: Vec<QhyLightspeed> = cameras.into_iter().map(QhyLightspeed::from).collect();
    log::info!("{} camera(s) found", devices.len());

    let (state_tx, state_rx) = mpsc::sync_channel(64);
    let handles = spawn_devices(devices, Duration::from_millis(1000), state_tx);

    let cmd_map: HashMap<String, mpsc::SyncSender<Command>> = handles
        .iter()
        .map(|h| (h.camera_id.clone(), h.cmd_tx.clone()))
        .collect();

    let mut mqtt_opts = MqttOptions::new("qhy-lightspeed", &args.host, args.port);
    mqtt_opts.set_keep_alive(Duration::from_secs(30));
    mqtt_opts.set_max_packet_size(1024 * 10000, 1024 * 10000);
    let (client, mut connection) = Client::new(mqtt_opts, 16);

    client
        .subscribe("devices/+/expose", QoS::AtLeastOnce)
        .expect("subscribe failed");
    client
        .subscribe("devices/+/set", QoS::AtLeastOnce)
        .expect("subscribe failed");

    std::thread::spawn(move || {
        for event in connection.iter() {
            match event {
                Ok(Event::Incoming(Packet::Publish(p))) => {
                    dispatch(&p.topic, &p.payload, &cmd_map);
                }
                Err(e) => eprintln!("MQTT: {e}"),
                _ => {}
            }
        }
    });

    for msg in state_rx {
        if let Err(e) = client.publish(&msg.topic, QoS::AtMostOnce, false, msg.payload.as_bytes()) {
            eprintln!("publish error: {e}");
        }
    }

    for handle in handles {
        let _ = handle.thread.join();
    }
}
