use astrotools::{
    Lightspeed, LightspeedError,
    properties::{Permission, Prop, Property, RangeProperty},
    types::{DevType, DeviceType},
};
use libqhy::QhyCcd;

pub struct QhyLightspeed {
    camera: QhyCcd,
    connected: Property<bool>,
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

    pub fn set_gain(&mut self, value: f64) -> Result<(), LightspeedError> {
        self.gain.update(value)?;
        self.camera
            .set_gain(value)
            .map_err(|_| LightspeedError::DeviceConnectionError)
    }

    pub fn set_offset(&mut self, value: f64) -> Result<(), LightspeedError> {
        self.offset.update(value)?;
        self.camera
            .set_offset(value)
            .map_err(|_| LightspeedError::DeviceConnectionError)
    }

    pub fn set_exposure(&mut self, value: f64) -> Result<(), LightspeedError> {
        self.exposure.update(value)?;
        self.camera
            .set_exposure(value)
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

        Self {
            gain: RangeProperty::new(gain_cur, Permission::ReadWrite, gain_min, gain_max),
            offset: RangeProperty::new(offset_cur, Permission::ReadWrite, offset_min, offset_max),
            exposure: RangeProperty::new(1000.0, Permission::ReadWrite, 1.0, 3_600_000_000.0),
            temperature,
            pixel_size: Property::new(pixel_size, Permission::ReadOnly),
            connected: Property::new(true, Permission::ReadWrite),
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

    fn update_property<T>(&mut self, _prop_name: &str, _val: T) -> Result<(), LightspeedError> {
        todo!()
    }
}
