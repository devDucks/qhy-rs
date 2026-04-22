use astrotools::{
    Lightspeed, LightspeedError,
    properties::{Permission, Property, RangeProperty},
    types::{DevType, DeviceType},
};
use libqhy::{QhyCcd, raw::CameraHandle};

pub struct QhyLightspeed {
    camera_id: String,
    handle: CameraHandle,
    connected: Property<bool>,
    exposure: RangeProperty<f64>,
    gain: RangeProperty<f64>,
    offset: RangeProperty<f64>,
    temperature: Property<f64>,
}

impl From<QhyCcd> for QhyLightspeed {
    fn from(cam: QhyCcd) -> Self {
        Self {
            camera_id: cam.id().to_string(),
            handle: cam.handle,
            connected: Property::new(true, Permission::ReadWrite),
            exposure: RangeProperty::new(1000.0, Permission::ReadWrite, 1.0, 3_600_000_000.0),
            gain: RangeProperty::new(0.0, Permission::ReadWrite, 0.0, 100.0),
            offset: RangeProperty::new(0.0, Permission::ReadWrite, 0.0, 255.0),
            temperature: Property::new(0.0, Permission::ReadOnly),
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
        todo!()
    }

    fn update_property<T>(&mut self, _prop_name: &str, _val: T) -> Result<(), LightspeedError> {
        todo!()
    }
}
