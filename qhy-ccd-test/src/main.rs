use libqhy::*;
use log::{debug, error};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    let _ = init_resources();
    debug!("Initializing QHY resources");
    let cam_num = get_num_of_connected_cameras();
    debug!("Found {} cameras", cam_num);

    for idx in 0..cam_num {
        if let Ok(id) = get_camera_id(idx) {
            debug!("Found camera {}", id);
            if let Ok(handle) = open_camera(&id) {
                debug!("Opened sucessfully camera {}", id);
                if let Ok(fw) = read_camera_fw(&handle) {
                    debug!("FW version for camera {}: {}", id, fw)
                } else {
                    error!("Couldn't read the firmware version of camera {}", id);
                }

                if let Ok(sdk) = read_sdk_version() {
                    debug!("SDK version for camera {}: {}", id, sdk)
                } else {
                    error!("Couldn't read the SDK version of camera {}", id);
                }

                // We are done, close the camera
                if let Ok(()) = close_camera(handle) {
                    debug!("Camera {} successfully closed", id);
                } else {
                    error!("Couldn't close camera {}", id);
                }
            } else {
                error!("Failed to get handle for camera {}", id)
            }
        } else {
            error!("Couldn't read camera ID for camera {}", idx);
        }
    }

    if release_resources().is_ok() {
        debug!("All QHY resources released");
    } else {
        error!("Unable to release QHY resources, unplug them");
    }
}
