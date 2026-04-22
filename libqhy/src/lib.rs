pub mod raw;
pub mod types;

use raw::{CameraHandle, ChipInfo};

pub struct QhyCcd {
    id: String,
    handle: CameraHandle,
    pub chip_info: ChipInfo,
}

impl QhyCcd {
    pub fn id(&self) -> &str {
        &self.id
    }
}

pub fn init_sdk() -> Vec<QhyCcd> {
    if raw::init_resources().is_err() {
        return vec![];
    }

    let count = raw::get_num_of_connected_cameras();
    let mut cameras = Vec::with_capacity(count as usize);

    for idx in 0..count {
        let id = match raw::get_camera_id(idx) {
            Ok(id) => id,
            Err(_) => continue,
        };
        let handle = match raw::open_camera(&id) {
            Ok(h) => h,
            Err(_) => continue,
        };
        let chip_info = match raw::read_chip_info(&handle) {
            Ok(info) => info,
            Err(_) => continue,
        };
        cameras.push(QhyCcd {
            id,
            handle,
            chip_info,
        });
    }

    cameras
}
