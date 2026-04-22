use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::os::raw::c_uint;

pub use crate::types::{AvailableControls, ControlId, ControlValue};

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

pub struct ChipInfo {
    pub chip_width: f64,
    pub chip_height: f64,
    pub pixel_width: f64,
    pub pixel_height: f64,
    pub image_width: u32,
    pub image_height: u32,
    pub bpp: u32,
}

impl Display for ChipInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "Chip width:{} Chip height:{} Px widht:{} Px height:{} Img width:{} Img height:{} Depth:{}",
            self.chip_width,
            self.chip_height,
            self.pixel_width,
            self.pixel_height,
            self.image_width,
            self.image_height,
            self.bpp
        )
    }
}

pub fn read_chip_info(handle: &CameraHandle) -> Result<ChipInfo, QHYError> {
    let mut chip_width = 0f64;
    let mut chip_height = 0f64;
    let mut pixel_width = 0f64;
    let mut pixel_height = 0f64;
    let mut image_width = 0u32;
    let mut image_height = 0u32;
    let mut bpp = 0u32;

    check_error(unsafe {
        libqhy_sys::camera::GetQHYCCDChipInfo(
            handle.as_ptr(),
            &mut chip_width,
            &mut chip_height,
            &mut image_width,
            &mut image_height,
            &mut pixel_width,
            &mut pixel_height,
            &mut bpp,
        )
    })?;

    Ok(ChipInfo {
        chip_width,
        chip_height,
        pixel_width,
        pixel_height,
        image_width,
        image_height,
        bpp,
    })
}

pub fn get_image_buffer_size(handle: &CameraHandle) -> u32 {
    unsafe { libqhy_sys::camera::GetQHYCCDMemLength(handle.as_ptr()) }
}

pub fn get_number_of_read_modes(handle: &CameraHandle) -> Result<u32, QHYError> {
    let mut num_modes = 0u32;
    check_error(unsafe {
        libqhy_sys::camera::GetQHYCCDNumberOfReadModes(handle.as_ptr(), &mut num_modes)
    })?;
    Ok(num_modes)
}

pub fn get_read_mode_name(handle: &CameraHandle, mode: u32) -> Result<String, QHYError> {
    let mut buf = [0i8; 64];
    check_error(unsafe {
        libqhy_sys::camera::GetQHYCCDReadModeName(handle.as_ptr(), mode, buf.as_mut_ptr())
    })?;
    let name = unsafe { CStr::from_ptr(buf.as_ptr()) }
        .to_string_lossy()
        .into_owned();
    Ok(name)
}

pub fn get_read_mode_resolution(handle: &CameraHandle, mode: u32) -> Result<(u32, u32), QHYError> {
    let mut width = 0u32;
    let mut height = 0u32;
    check_error(unsafe {
        libqhy_sys::camera::GetQHYCCDReadModeResolution(
            handle.as_ptr(),
            mode,
            &mut width,
            &mut height,
        )
    })?;
    Ok((width, height))
}

pub fn init_camera(handle: &CameraHandle) -> Result<(), QHYError> {
    check_error(unsafe { libqhy_sys::camera::InitQHYCCD(handle.as_ptr()) })
}

pub enum StreamMode {
    SingleFrame = 0,
    Live = 1,
}

pub fn set_stream_mode(handle: &CameraHandle, mode: StreamMode) -> Result<(), QHYError> {
    check_error(unsafe { libqhy_sys::camera::SetQHYCCDStreamMode(handle.as_ptr(), mode as u8) })
}

pub fn set_read_mode(handle: &CameraHandle, mode: u32) -> Result<(), QHYError> {
    check_error(unsafe { libqhy_sys::camera::SetQHYCCDReadMode(handle.as_ptr(), mode) })
}

pub fn enable_message(enable: bool) {
    unsafe { libqhy_sys::camera::EnableQHYCCDMessage(enable) }
}

pub fn is_control_available(handle: &CameraHandle, control: ControlId) -> bool {
    unsafe { libqhy_sys::camera::IsQHYCCDControlAvailable(handle.as_ptr(), control as i32) == 0 }
}

/// Set any of the available `ControlId` to a given value of type f64.
/// The function itself doesn't check if the ControlId exists for the given
/// camera so the caller, if interested into not receving an error, shall
/// first fetch available controls using [`get_available_controls`]
/// and call set_param using the available one returned into [`AvailableControls`]
pub fn set_param(handle: &CameraHandle, control: ControlId, value: f64) -> Result<(), QHYError> {
    check_error(unsafe {
        libqhy_sys::camera::SetQHYCCDParam(handle.as_ptr(), control as i32, value)
    })
}

pub fn exp_single_frame(handle: &CameraHandle) -> Result<(), QHYError> {
    let ret = unsafe { libqhy_sys::camera::ExpQHYCCDSingleFrame(handle.as_ptr()) };
    // The SDK docs say any non-QHYCCD_ERROR return is success for this call.
    if ret == libqhy_sys::camera::QHYCCD_ERROR {
        Err(QHYError {})
    } else {
        Ok(())
    }
}

pub struct FrameInfo {
    pub width: u32,
    pub height: u32,
    pub bpp: u32,
    pub channels: u32,
}

pub fn get_single_frame(handle: &CameraHandle, buf: &mut [u8]) -> Result<FrameInfo, QHYError> {
    let mut width = 0u32;
    let mut height = 0u32;
    let mut bpp = 0u32;
    let mut channels = 0u32;
    check_error(unsafe {
        libqhy_sys::camera::GetQHYCCDSingleFrame(
            handle.as_ptr(),
            &mut width,
            &mut height,
            &mut bpp,
            &mut channels,
            buf.as_mut_ptr(),
        )
    })?;
    Ok(FrameInfo {
        width,
        height,
        bpp,
        channels,
    })
}

pub fn set_bin_mode(handle: &CameraHandle, wbin: u32, hbin: u32) -> Result<(), QHYError> {
    check_error(unsafe { libqhy_sys::camera::SetQHYCCDBinMode(handle.as_ptr(), wbin, hbin) })
}

pub fn set_resolution(
    handle: &CameraHandle,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<(), QHYError> {
    check_error(unsafe {
        libqhy_sys::camera::SetQHYCCDResolution(handle.as_ptr(), x, y, width, height)
    })
}

pub fn get_available_controls(handle: &CameraHandle) -> AvailableControls {
    use strum::IntoEnumIterator;

    let mut map = HashMap::new();
    for control in ControlId::iter() {
        if !is_control_available(handle, control) {
            continue;
        }
        let mut min = 0f64;
        let mut max = 0f64;
        let mut step = 0f64;
        if unsafe {
            libqhy_sys::camera::GetQHYCCDParamMinMaxStep(
                handle.as_ptr(),
                control as i32,
                &mut min,
                &mut max,
                &mut step,
            )
        } != 0
        {
            continue;
        }
        let current =
            unsafe { libqhy_sys::camera::GetQHYCCDParam(handle.as_ptr(), control as i32) };
        map.insert(
            control,
            ControlValue {
                min,
                max,
                step,
                current,
            },
        );
    }
    map
}
