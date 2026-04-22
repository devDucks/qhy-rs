pub mod raw;
pub mod types;

use raw::{CameraHandle, ChipInfo, FwVersion, SDKVersion};
use types::AvailableControls;

pub struct QhyCcd {
    id: String,
    pub handle: CameraHandle,
    pub chip_info: ChipInfo,
    pub controls: AvailableControls,
    fw_ver: FwVersion,
}

impl QhyCcd {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn fw_version(&self) -> String {
        self.fw_ver.to_string()
    }

    pub fn fw_major(&self) -> u16 {
        self.fw_ver.year
    }

    pub fn fw_minor(&self) -> u8 {
        self.fw_ver.month
    }

    pub fn fw_patch(&self) -> u8 {
        self.fw_ver.day
    }

    pub fn set_bin(&self, bin: u32) -> Result<(), raw::QHYError> {
        raw::set_bin_mode(&self.handle, bin, bin)
    }
}

pub struct SdkContext {
    pub cameras: Vec<QhyCcd>,
    sdk_ver: SDKVersion,
}

impl SdkContext {
    pub fn sdk_version(&self) -> String {
        self.sdk_ver.to_string()
    }

    pub fn sdk_major(&self) -> u16 {
        self.sdk_ver.year
    }

    pub fn sdk_minor(&self) -> u8 {
        self.sdk_ver.month
    }

    pub fn sdk_patch(&self) -> u8 {
        self.sdk_ver.day
    }
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
    SdkVersionReadFailed,
}

pub fn init_sdk() -> Result<SdkContext, SdkError> {
    raw::init_resources().map_err(|_| SdkError::InitFailed)?;

    let sdk_ver = raw::read_sdk_version().map_err(|_| SdkError::SdkVersionReadFailed)?;

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
        let controls = raw::get_available_controls(&handle);
        let fw_ver = match raw::read_camera_fw(&handle) {
            Ok(v) => v,
            Err(_) => continue,
        };
        cameras.push(QhyCcd {
            id,
            handle,
            chip_info,
            controls,
            fw_ver,
        });
    }

    Ok(SdkContext { cameras, sdk_ver })
}
