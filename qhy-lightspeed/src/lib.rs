pub mod runtime;

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};

use serde::{Deserialize, Serialize, Serializer};

use astrotools::{
    Lightspeed, LightspeedError,
    properties::{Permission, Prop, PropValue, Property, PropertyErrorType, RangeProperty},
    types::{DevType, DeviceType},
};
use libqhy::QhyCcd;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum FrameType {
    Light,
    Dark,
    Bias,
    Flat,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExposureValue {
    pub duration_us: u64,
    pub frame_type: FrameType,
}

fn serialize_base64<S: Serializer>(data: &[u8], s: S) -> Result<S::Ok, S::Error> {
    use base64::{Engine, engine::general_purpose::STANDARD};
    s.serialize_str(&STANDARD.encode(data))
}

#[derive(Serialize)]
pub struct Frame {
    pub frame_type: FrameType,
    pub width: u32,
    pub height: u32,
    pub bpp: u32,
    pub channels: u32,
    #[serde(serialize_with = "serialize_base64")]
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize)]
pub enum ExposureState {
    Idle,
    Exposing(FrameType),
    ReadingOut,
}

#[derive(Serialize)]
pub struct QhyLightspeed {
    #[serde(skip)]
    camera: Arc<QhyCcd>,
    #[serde(skip)]
    frame_buf: Vec<u8>,
    exposure_state: ExposureState,
    #[serde(skip)]
    exposure_done: Option<Arc<AtomicBool>>,
    #[serde(skip)]
    readout_rx: Option<mpsc::Receiver<(Option<Frame>, Vec<u8>)>>,
    #[serde(skip)]
    exposure: RangeProperty<f64>,
    gain: RangeProperty<f64>,
    offset: RangeProperty<f64>,
    temperature: Option<Property<f64>>,
    pixel_size: Property<f64>,
}

impl QhyLightspeed {
    pub fn camera_id(&self) -> &str {
        self.camera.id()
    }

    pub fn fw_version(&self) -> String {
        self.camera.fw_version()
    }

    fn set_gain(&mut self, value: f64) -> Result<(), LightspeedError> {
        self.gain.update(value)?;
        self.camera
            .set_gain(value)
            .map_err(|_| LightspeedError::DeviceConnectionError)
    }

    fn set_offset(&mut self, value: f64) -> Result<(), LightspeedError> {
        self.offset.update(value)?;
        self.camera
            .set_offset(value)
            .map_err(|_| LightspeedError::DeviceConnectionError)
    }

    fn set_exposure(&mut self, value: f64) -> Result<(), LightspeedError> {
        self.exposure.update(value)?;
        self.camera
            .set_exposure(value)
            .map_err(|_| LightspeedError::DeviceConnectionError)
    }

    fn set_bin(&mut self, bin: u32) -> Result<(), LightspeedError> {
        self.camera
            .set_bin(bin)
            .map_err(|_| LightspeedError::DeviceConnectionError)
    }

    pub fn start_exposure(&mut self, val: ExposureValue) -> Result<(), LightspeedError> {
        self.camera
            .set_exposure(val.duration_us as f64)
            .map_err(|_| LightspeedError::DeviceConnectionError)?;

        let done = Arc::new(AtomicBool::new(false));
        self.exposure_done = Some(Arc::clone(&done));
        self.exposure_state = ExposureState::Exposing(val.frame_type);

        let camera = Arc::clone(&self.camera);
        std::thread::spawn(move || {
            log::info!("ExpQHYCCDSingleFrame: start");
            match camera.start_exposure() {
                Ok(libqhy::raw::ExpResult::Exposing) => {
                    while camera.is_exposing() {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }
                Ok(libqhy::raw::ExpResult::ReadDirectly) => {}
                Err(_) => return,
            }
            log::info!("ExpQHYCCDSingleFrame: done");
            done.store(true, Ordering::Release);
        });

        Ok(())
    }

    pub fn poll_exposure(&mut self) {
        if let ExposureState::Exposing(ft) = self.exposure_state {
            if self
                .exposure_done
                .as_ref()
                .is_some_and(|f| f.load(Ordering::Acquire))
            {
                self.start_readout(ft);
            }
        }
    }

    fn start_readout(&mut self, ft: FrameType) {
        let camera = Arc::clone(&self.camera);
        let buf = std::mem::take(&mut self.frame_buf);
        let (tx, rx) = mpsc::sync_channel(1);
        self.readout_rx = Some(rx);
        self.exposure_state = ExposureState::ReadingOut;

        std::thread::spawn(move || {
            log::info!("GetQHYCCDSingleFrame: start");
            let mut buf = buf;
            let result = match camera.read_frame(&mut buf) {
                Ok(info) => {
                    log::info!(
                        "GetQHYCCDSingleFrame: complete {}x{} @{}bpp",
                        info.width,
                        info.height,
                        info.bpp
                    );
                    let data_len = info.width as usize
                        * info.height as usize
                        * (info.bpp as usize).div_ceil(8);
                    Some(Frame {
                        frame_type: ft,
                        width: info.width,
                        height: info.height,
                        bpp: info.bpp,
                        channels: info.channels,
                        data: buf[..data_len].to_vec(),
                    })
                }
                Err(_) => {
                    log::error!("GetQHYCCDSingleFrame failed; dropping exposure");
                    None
                }
            };
            tx.send((result, buf)).ok();
        });
    }

    pub fn poll_readout(&mut self) -> Option<Frame> {
        let rx = self.readout_rx.as_ref()?;
        let (frame, buf) = rx.try_recv().ok()?;
        self.frame_buf = buf;
        self.readout_rx = None;
        self.exposure_state = ExposureState::Idle;
        frame
    }
}

impl From<QhyCcd> for QhyLightspeed {
    fn from(cam: QhyCcd) -> Self {
        let temperature = cam
            .temperature()
            .map(|t| Property::new(t, Permission::ReadOnly));
        let (gain_min, gain_max) = cam.gain_range().unwrap_or((0.0, 100.0));
        let gain_cur = cam.gain().unwrap_or(0.0);
        let (offset_min, offset_max) = cam.offset_range().unwrap_or((0.0, 255.0));
        let offset_cur = cam.offset().unwrap_or(0.0);
        let pixel_size = cam.chip_info.pixel_width;
        let buf_size = cam.image_buffer_size() as usize;

        Self {
            gain: RangeProperty::new(gain_cur, Permission::ReadWrite, gain_min, gain_max),
            offset: RangeProperty::new(offset_cur, Permission::ReadWrite, offset_min, offset_max),
            exposure: RangeProperty::new(1000.0, Permission::ReadWrite, 1.0, 3_600_000_000.0),
            temperature,
            pixel_size: Property::new(pixel_size, Permission::ReadOnly),
            frame_buf: vec![0u8; buf_size],
            exposure_state: ExposureState::Idle,
            exposure_done: None,
            readout_rx: None,
            camera: Arc::new(cam),
        }
    }
}

impl DevType for QhyLightspeed {
    fn dev_type(&self) -> DeviceType {
        DeviceType::Ccd
    }
}

impl Lightspeed for QhyLightspeed {
    fn sync_state(&mut self) {
        let live_temp = self.camera.temperature();
        let live_gain = self.camera.gain();
        let live_offset = self.camera.offset();

        if let (Some(prop), Some(val)) = (&mut self.temperature, live_temp) {
            let _ = prop.update_int(val);
        }
        if let Some(val) = live_gain {
            let _ = self.gain.update_int(val);
        }
        if let Some(val) = live_offset {
            let _ = self.offset.update_int(val);
        }
    }

    fn update_property(&mut self, prop_name: &str, val: PropValue) -> Result<(), LightspeedError> {
        match prop_name {
            "gain" => self.set_gain(f64::try_from(val)?),
            "offset" => self.set_offset(f64::try_from(val)?),
            "exposure" => self.set_exposure(f64::try_from(val)?),
            "bin" => self.set_bin(u32::try_from(val)?),
            _ => Err(LightspeedError::PropertyError(
                PropertyErrorType::InvalidValue,
            )),
        }
    }
}
