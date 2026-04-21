use std::ffi::{CStr, CString};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::os::raw::c_uint;

pub struct QHYError {}

pub struct CameraHandle(*mut libqhy_sys::camera::qhyccd_handle);

impl Drop for CameraHandle {
    fn drop(&mut self) {
        unsafe { libqhy_sys::camera::CloseQHYCCD(self.0) };
    }
}

impl CameraHandle {
    pub fn as_ptr(&self) -> *mut libqhy_sys::camera::qhyccd_handle {
        self.0
    }
}

fn check_error(err: c_uint) -> Result<(), QHYError> {
    if err != 0 {
        return Err(QHYError {});
    }

    Ok(())
}

pub fn get_num_of_connected_cameras() -> c_uint {
    unsafe { libqhy_sys::camera::ScanQHYCCD() }
}

pub fn get_camera_id(idx: c_uint) -> Result<String, QHYError> {
    let mut buf = [0i8; 64];
    check_error(unsafe { libqhy_sys::camera::GetQHYCCDId(idx, buf.as_mut_ptr()) })?;
    let id = unsafe { CStr::from_ptr(buf.as_ptr()) }
        .to_string_lossy()
        .into_owned();
    Ok(id)
}

pub fn init_resources() -> Result<(), QHYError> {
    check_error(unsafe { libqhy_sys::camera::InitQHYCCDResource() })
}

pub fn release_resources() -> Result<(), QHYError> {
    check_error(unsafe { libqhy_sys::camera::ReleaseQHYCCDResource() })
}

pub fn open_camera(id: &str) -> Result<CameraHandle, QHYError> {
    let c_id = CString::new(id).map_err(|_| QHYError {})?;
    let handle = unsafe { libqhy_sys::camera::OpenQHYCCD(c_id.as_ptr() as *mut _) };
    if handle.is_null() {
        Err(QHYError {})
    } else {
        Ok(CameraHandle(handle))
    }
}

pub fn close_camera(handle: CameraHandle) -> Result<(), QHYError> {
    check_error(unsafe { libqhy_sys::camera::CloseQHYCCD(handle.as_ptr()) })
}

pub struct FwVersion {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl Display for FwVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}.{}.{}", self.year, self.month, self.day)
    }
}

pub struct SDKVersion {
    year: u16,
    month: u8,
    day: u8,
    subday: u8,
}

impl Display for SDKVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}.{}.{}.{}",
            self.year, self.month, self.day, self.subday
        )
    }
}

pub fn read_camera_fw(handle: &CameraHandle) -> Result<FwVersion, QHYError> {
    let mut buf = [0u8; 32];
    check_error(unsafe {
        libqhy_sys::camera::GetQHYCCDFWVersion(handle.as_ptr(), buf.as_mut_ptr())
    })?;

    // WINUSB cameras encode the year differently from CYUSB cameras; both
    // produce a 2-digit suffix that is prefixed with "20" to form the full year.
    let year_suffix = if (buf[0] >> 4) <= 9 {
        (buf[0] >> 4) + 0x10
    } else {
        buf[0] >> 4
    };

    Ok(FwVersion {
        year: 2000 + year_suffix as u16,
        month: buf[0] & 0x0f,
        day: buf[1],
    })
}

pub fn read_sdk_version() -> Result<SDKVersion, QHYError> {
    let mut year = 0;
    let mut month = 0;
    let mut day = 0;
    let mut subday = 0;

    check_error(unsafe {
        libqhy_sys::camera::GetQHYCCDSDKVersion(&mut year, &mut month, &mut day, &mut subday)
    })?;

    Ok(SDKVersion {
        year: 2000 + year as u16,
        month: month as u8,
        day: day as u8,
        subday: subday as u8,
    })
}
