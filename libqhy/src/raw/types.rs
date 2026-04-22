use std::collections::HashMap;
use strum::{EnumIter, FromRepr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, FromRepr)]
#[repr(u32)]
pub enum ControlId {
    Brightness = 0,
    Contrast = 1,
    Wbr = 2,
    Wbb = 3,
    Wbg = 4,
    Gamma = 5,
    Gain = 6,
    Offset = 7,
    Exposure = 8,
    Speed = 9,
    TransferBit = 10,
    Channels = 11,
    UsbTraffic = 12,
    RowNoiseRe = 13,
    CurTemp = 14,
    CurPwm = 15,
    ManualPwm = 16,
    CfwPort = 17,
    Cooler = 18,
    St4Port = 19,
    Color = 20,
    Bin1x1Mode = 21,
    Bin2x2Mode = 22,
    Bin3x3Mode = 23,
    Bin4x4Mode = 24,
    MechanicalShutter = 25,
    TriggerInterface = 26,
    TecOverprotectInterface = 27,
    SignalClampInterface = 28,
    FineToneInterface = 29,
    ShutterMotorHeatingInterface = 30,
    CalibrateFpnInterface = 31,
    ChipTemperatureSensorInterface = 32,
    UsbReadoutSlowestInterface = 33,
    Bits8 = 34,
    Bits16 = 35,
    Gps = 36,
    IgnoreOverscanInterface = 37,
    // 38 is not used (QHYCCD_3A_AUTOBALANCE was moved to 1024)
    AutoExposure3a = 39,
    AutoFocus3a = 40,
    Ampv = 41,
    VirtualCamera = 42,
    ViewMode = 43,
    CfwSlotsNum = 44,
    IsExposingDone = 45,
    ScreenStretchB = 46,
    ScreenStretchW = 47,
    Ddr = 48,
    LightPerformanceMode = 49,
    Qhy5IiGuideMode = 50,
    DdrBufferCapacity = 51,
    DdrBufferReadThreshold = 52,
    DefaultGain = 53,
    DefaultOffset = 54,
    OutputDataActualBits = 55,
    OutputDataAlignment = 56,
    SingleFrameMode = 57,
    LiveVideoMode = 58,
    IsColor = 59,
    HardwareFrameCounter = 60,
    MaxIdError = 61,
    Humidity = 62,
    Pressure = 63,
    VacuumPump = 64,
    SensorChamberCyclePump = 65,
    Bits32 = 66,
    SensorUlvoStatus = 67,
    SensorPhaseReTrain = 68,
    InitConfigFromFlash = 69,
    TriggerMode = 70,
    TriggerOut = 71,
    BurstMode = 72,
    SpeakerLedAlarm = 73,
    WatchDogFpga = 74,
    Bin6x6Mode = 75,
    Bin8x8Mode = 76,
    GlobalSensorGpsLed = 77,
    ImgProc = 78,
    RemoveRbi = 79,
    GlobalReset = 80,
    FrameDetect = 81,
    GainDbConversion = 82,
    CurveSystemGain = 83,
    CurveFullWell = 84,
    CurveReadoutNoise = 85,
    UseAverageBinning = 86,
    OutsidePumpV2 = 87,
    AutoExposure = 88,
    AutoExpTargetBrightness = 89,
    AutoExpSampleArea = 90,
    AutoExpMaxMs = 91,
    AutoExpGainMax = 92,
    ErrorLed = 93,
    MaxId = 94,
    AutoWhiteBalance = 1024,
    ImageStabilization = 1025,
    GainDb = 1026,
    Dpc = 1027,
    DpcValue = 1028,
    Hdr = 1029,
    HdrLk = 1030,
    HdrLb = 1031,
    HdrX = 1032,
    HdrShowKb = 1033,
}

impl TryFrom<u32> for ControlId {
    type Error = u32;

    fn try_from(v: u32) -> Result<Self, Self::Error> {
        Self::from_repr(v).ok_or(v)
    }
}

impl From<ControlId> for u32 {
    fn from(id: ControlId) -> u32 {
        id as u32
    }
}

#[derive(Debug, Clone)]
pub struct ControlValue {
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub current: f64,
}

pub type AvailableControls = HashMap<ControlId, ControlValue>;
