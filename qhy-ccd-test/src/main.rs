use libqhy::*;
use log::{debug, error};

fn probe_camera(idx: u32) -> Result<(), libqhy::QHYError> {
    let id = get_camera_id(idx)?;
    debug!("Found camera {}", id);
    let handle = open_camera(&id)?;
    debug!("Opened camera {} successfully", id);

    match read_camera_fw(&handle) {
        Ok(fw) => debug!("FW version for camera {}: {}", id, fw),
        Err(_) => error!("Couldn't read FW version of camera {}", id),
    }
    match read_sdk_version() {
        Ok(sdk) => debug!("SDK version for camera {}: {}", id, sdk),
        Err(_) => error!("Couldn't read SDK version of camera {}", id),
    }

    match read_chip_info(&handle) {
        Ok(info) => debug!("Chip info => {}", info),
        Err(_) => error!("Couldn't read chip info of camera {}", id),
    }

    debug!(
        "Buffer to fit an image: {} bytes",
        get_image_buffer_size(&handle)
    );

    match get_number_of_read_modes(&handle) {
        Ok(n) => {
            debug!("Number of read modes for camera {}: {}", id, n);
            for mode in 0..n {
                match get_read_mode_name(&handle, mode) {
                    Ok(name) => debug!("  Read mode {}: {}", mode, name),
                    Err(_) => error!("Couldn't read mode name {} of camera {}", mode, id),
                }
            }
        }
        Err(_) => error!("Couldn't get number of read modes of camera {}", id),
    }

    let controls = get_available_controls(&handle);
    debug!("Available controls for camera {}:", id);
    let mut sorted: Vec<_> = controls.iter().collect();
    sorted.sort_by_key(|(k, _)| **k as u32);
    for (ctrl, val) in sorted {
        debug!(
            "  {:?}: current={} min={} max={} step={}",
            ctrl, val.current, val.min, val.max, val.step
        );
    }

    match close_camera(handle) {
        Ok(()) => debug!("Camera {} successfully closed", id),
        Err(_) => error!("Couldn't close camera {}", id),
    }
    Ok(())
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    let _ = init_resources();
    debug!("Initializing QHY resources");
    let cam_num = get_num_of_connected_cameras();
    debug!("Found {} cameras", cam_num);

    for idx in 0..cam_num {
        if let Err(_) = probe_camera(idx) {
            error!("Failed to probe camera {}", idx);
        }
    }

    if release_resources().is_ok() {
        debug!("All QHY resources released");
    } else {
        error!("Unable to release QHY resources, unplug them");
    }
}
