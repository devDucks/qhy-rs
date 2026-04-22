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

pub struct SdkContext {
    pub cameras: Vec<QhyCcd>,
}

impl Drop for SdkContext {
    fn drop(&mut self) {
        self.cameras.clear();
        let _ = raw::release_resources();
    }
}

#[derive(Debug)]
pub enum SdkError {
    InitFailed,
}

pub fn init_sdk() -> Result<SdkContext, SdkError> {
    raw::init_resources().map_err(|_| SdkError::InitFailed)?;

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

    Ok(SdkContext { cameras })
}
