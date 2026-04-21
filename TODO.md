# QHY-RS TODO

Tracks which SDK API functions have been wrapped in `libqhy/src/lib.rs`.

## API Coverage

### Core lifecycle

- [x] `InitQHYCCDResource` → `init_resources()`
- [x] `ScanQHYCCD` → `get_num_of_connected_cameras()`
- [x] `GetQHYCCDId` → `get_camera_id()`
- [x] `OpenQHYCCD` → `open_camera()`
- [x] `CloseQHYCCD` → `close_camera()` / `CameraHandle::drop()`
- [x] `ReleaseQHYCCDResource` → `release_resources()`
- [x] `GetQHYCCDFWVersion` → `read_camera_fw()`
- [x] `GetQHYCCDSDKVersion` → `read_sdk_version()`
- [x] `GetQHYCCDChipInfo` → `read_chip_info()`
- [x] `GetQHYCCDMemLength` → `get_image_buffer_size()`

---

## Not Yet Implemented

### Core setup (implement first — required for any capture workflow)

- [x] `InitQHYCCD` → `init_camera()`
- [x] `SetQHYCCDStreamMode` → `set_stream_mode()`
- [x] `GetQHYCCDNumberOfReadMode` → `get_number_of_read_modes()`
- [x] `GetQHYCCDReadModeName` → `get_read_mode_name()`
- [x] `GetQHYCCDReadModeResolution` — get image resolution for a given readout mode
- [x] `SetQHYCCDReadMode` → `set_read_mode()`

### Image area & format

- [ ] `GetQHYCCDEffectiveArea` — get valid image area (start coords + size)
- [ ] `GetQHYCCDOverScanArea` — get overscan area (start coords + size)
- [ ] `SetQHYCCDBinMode` — set horizontal/vertical binning (1x1, 2x2, etc.)
- [ ] `SetQHYCCDResolution` — set ROI (start x/y + width/height)
- [ ] `GetQHYCCDCurrentROI` — read back current ROI settings
- [ ] `SetQHYCCDDebayerOnOff` — enable/disable debayering for colour cameras

### Camera parameter control

- [x] `IsQHYCCDControlAvailable` — check if a `CONTROL_ID` feature is supported
- [x] `GetQHYCCDParamMinMaxStep` — get min/max/step for a `CONTROL_ID` parameter
- [x] `GetQHYCCDParam` — read a camera parameter by `CONTROL_ID`
- [x] `SetQHYCCDParam` — write a camera parameter by `CONTROL_ID`

### Capture

- [x] `ExpQHYCCDSingleFrame` — start a single-frame exposure
- [x] `GetQHYCCDSingleFrame` — retrieve image data after single-frame exposure
- [ ] `CancelQHYCCDExposingAndReadout` — abort exposure and readout
- [ ] `CancelQHYCCDExposing` — abort exposure only (WINUSB still needs readout)
- [ ] `BeginQHYCCDLive` — start continuous/live capture
- [ ] `GetQHYCCDLiveFrame` — retrieve one frame from the live stream
- [ ] `StopQHYCCDLive` — stop continuous capture

### Temperature control

- [ ] `ControlQHYCCDTemp` — set cooler target temperature (equivalent to `SetQHYCCDParam(CONTROL_COOLER)`)

### Color Filter Wheel (CFW)

- [ ] `IsQHYCCDCFWPlugged` — check whether a filter wheel is connected
- [ ] `SendOrder2QHYCCDCFW` — rotate filter wheel to a target slot
- [ ] `GetQHYCCDCFWStatus` — read current filter wheel position

### Sensor info & gain utilities

- [ ] `GetQHYCCDSensorName` — get sensor model name (e.g. `IMX411`)
- [ ] `QHYCCD_DbGainToGainValue` — convert dB gain to SDK gain value
- [ ] `QHYCCD_GainValueToDbGain` — convert SDK gain value to dB
- [ ] `QHYCCD_curveFullWell` — look up full-well capacity curve at a given gain
- [ ] `QHYCCD_curveReadoutNoise` — look up readout noise curve at a given gain
- [ ] `QHYCCD_curveSystemGain` — look up system gain curve at a given gain

### Precise timing & rolling shutter

- [ ] `GetQHYCCDPreciseExposureInfo` — get pixel/line/frame periods and actual exposure time
- [ ] `GetQHYCCDRollingShutterEndOffset` — get per-row exposure time offset (for GPS timestamping)

### Debug & events

- [x] `EnableQHYCCDMessage` — enable/disable SDK debug output
- [ ] `EnableQHYCCDImageOSD` — overlay frame counter or GPS info on image
- [ ] `RegisterPnpEventIn` — register callback for camera connect events
- [ ] `RegisterPnpEventOut` — register callback for camera disconnect events

### Miscellaneous

- [ ] `SetQHYCCDTwoChannelCombineParameter` — calibrate high/low gain channel stitching
- [ ] `QHYCCDSensorPhaseReTrain` — retrain internal sensor phase (fixes fringe artefacts)
- [ ] `GetQHYCCDImageStabilizationGravity` — get stabilisation gravity centre position

### Burst mode

- [ ] `SetQHYCCDEnableLiveModeAntiRBI` — enable anti-RBI (alternating light/dark frames)
- [ ] `EnableQHYCCDBurstMode` — enable/disable burst sub-mode of live capture
- [ ] `EnableQHYCCDBurstCountFun` — enable/disable frame counting in burst mode
- [ ] `ResetQHYCCDFrameCounter` — reset frame counter to 0
- [ ] `SetQHYCCDBurstModeStartEnd` — set start/end frame for burst output
- [ ] `SetQHYCCDBurstIDLE` — put camera into idle state (paired with release)
- [ ] `ReleaseQHYCCDBurstIDLE` — release idle state and start burst output
- [ ] `SetQHYCCDBurstModePatchNumber` — set packet padding size for burst mode

### Trigger

- [ ] `GetQHYCCDTrigerInterfaceNumber` — query number of trigger interfaces
- [ ] `GetQHYCCDTrigerInterfaceName` — get name of a trigger interface
- [ ] `SetQHYCCDTrigerInterface` — select active trigger interface
- [ ] `SetQHYCCDTrigerMode` — set trigger mode (0 = disable external trigger)
- [ ] `SetQHYCCDTrigerFunction` — enable/disable external trigger input
- [ ] `EnableQHYCCDTrigerOut` — enable trigger-out waveform output
- [ ] `EnableQHYCCDTrigerOutA` — enable trigger-out in mode A
- [ ] `SendSoftTriger2QHYCCDCam` — send a software trigger signal to the camera
- [ ] `SetQHYCCDTrigerFilterOnOff` — enable/disable trigger debounce filter
- [ ] `SetQHYCCDTrigerFilterTime` — set trigger debounce filter time (ms)

### Frame integrity detection

- [ ] `SetQHYCCDFrameDetectOnOff` — enable/disable frame integrity checking
- [ ] `SetQHYCCDFrameDetectCode` — set the expected checksum value
- [ ] `SetQHYCCDFrameDetectPos` — set the position of the checksum in frame data

### GPS (QHY174-GPS only)

- [ ] `SetQHYCCDGPSVCOXFreq` — set VCOX frequency
- [ ] `SetQHYCCDGPSLedCalMode` — set LED calibration mode (off / slave / master)
- [ ] `SetQHYCCDGPSPOSA` — set LED pulse position A (shutter start)
- [ ] `SetQHYCCDGPSPOSB` — set LED pulse position B (shutter end)
- [ ] `SetQHYCCDGPSMasterSlave` — set master/slave GPS mode
- [ ] `SetQHYCCDGPSSlaveModeParameter` — set slave mode timing parameters
