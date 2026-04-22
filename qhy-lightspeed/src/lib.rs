pub mod runtime;

use serde::Serialize;

use astrotools::{
    Lightspeed, LightspeedError,
    properties::{Permission, Prop, PropValue, Property, PropertyErrorType, RangeProperty},
    types::{DevType, DeviceType},
};
use libqhy::QhyCcd;

#[derive(Serialize)]
pub struct QhyLightspeed {
    #[serde(skip)]
    camera: QhyCcd,
    connected: Property<bool>,
    #[serde(skip)]
    exposure: RangeProperty<f64>,
    gain: RangeProperty<f64>,
    offset: RangeProperty<f64>,
    temperature: Option<Property<f64>>,
    pixel_size: Property<f64>,
    is_exposing: Property<bool>,
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
        let is_exposing = cam.is_exposing();

        Self {
            gain: RangeProperty::new(gain_cur, Permission::ReadWrite, gain_min, gain_max),
            offset: RangeProperty::new(offset_cur, Permission::ReadWrite, offset_min, offset_max),
            exposure: RangeProperty::new(1000.0, Permission::ReadWrite, 1.0, 3_600_000_000.0),
            temperature,
            pixel_size: Property::new(pixel_size, Permission::ReadOnly),
            connected: Property::new(true, Permission::ReadWrite),
            is_exposing: Property::new(is_exposing, Permission::ReadOnly),
            camera: cam,
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
        let live_is_exposing = self.camera.is_exposing();

        if let (Some(prop), Some(val)) = (&mut self.temperature, live_temp) {
            let _ = prop.update_int(val);
        }
        if let Some(val) = live_gain {
            let _ = self.gain.update_int(val);
        }
        if let Some(val) = live_offset {
            let _ = self.offset.update_int(val);
        }
        let _ = self.is_exposing.update_int(live_is_exposing);
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
