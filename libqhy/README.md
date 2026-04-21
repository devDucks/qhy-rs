# libqhy

Safe Rust bindings to the [QHY CCD SDK](https://www.qhyccd.com/html/prepub/log_en.html#!log_en.md) for controlling QHY astronomical cameras.

## Add to your project

```toml
[dependencies]
libqhy = "0.1"
```

> **SDK requirement** — the crate links against `libqhyccd.so`. See the [top-level README](../README.md) for installation instructions on Arch Linux, Ubuntu/Debian, or manual setups.

## Quick examples

### List connected cameras

```rust
use libqhy::*;

fn main() -> Result<(), QHYError> {
    init_resources()?;

    let count = get_num_of_connected_cameras();
    println!("Found {} camera(s)", count);

    for idx in 0..count {
        let id = get_camera_id(idx)?;
        println!("  [{}] {}", idx, id);
    }

    release_resources()?;
    Ok(())
}
```

### Read available controls

```rust
use libqhy::*;

fn main() -> Result<(), QHYError> {
    init_resources()?;

    let id = get_camera_id(0)?;
    let handle = open_camera(&id)?;
    init_camera(&handle)?;

    let controls = get_available_controls(&handle);
    for (ctrl, val) in &controls {
        println!("{:?}: current={:.1}  [{:.1} … {:.1}  step {:.1}]",
            ctrl, val.current, val.min, val.max, val.step);
    }

    release_resources()?;
    Ok(())
}
```

### Take a single frame

```rust
use libqhy::*;

fn main() -> Result<(), QHYError> {
    init_resources()?;

    let id = get_camera_id(0)?;
    let handle = open_camera(&id)?;

    set_stream_mode(&handle, StreamMode::SingleFrame)?;
    set_read_mode(&handle, 0)?;
    init_camera(&handle)?;

    // 5-second exposure at gain 50, 16-bit depth
    set_param(&handle, ControlId::Exposure, 5_000_000.0)?;
    set_param(&handle, ControlId::Gain, 50.0)?;
    set_param(&handle, ControlId::Bits16, 1.0)?;

    let buf_size = get_image_buffer_size(&handle) as usize;
    let mut buf = vec![0u8; buf_size];

    exp_single_frame(&handle)?;
    let info = get_single_frame(&handle, &mut buf)?;

    println!("Got frame {}×{} @ {}bpp, {} channel(s)",
        info.width, info.height, info.bpp, info.channels);

    release_resources()?;
    Ok(())
}
```

## Full usage sketch

The typical lifecycle mirrors the underlying SDK:

```rust
use libqhy::*;

fn main() -> Result<(), QHYError> {
    // 1. Initialise SDK resources (USB subsystem, firmware loader, …)
    init_resources()?;

    // 2. Scan the bus and pick a camera by its string ID
    let count = get_num_of_connected_cameras();
    let id = get_camera_id(0)?;   // e.g. "QHY600M-xxxxxxxx"

    // 3. Open the camera — returns a RAII handle (closes on drop)
    let handle = open_camera(&id)?;

    // 4. Configure stream mode and read mode before init
    set_stream_mode(&handle, StreamMode::SingleFrame)?;
    set_read_mode(&handle, 0)?;

    // 5. Initialise the camera (uploads firmware / default config)
    init_camera(&handle)?;

    // 6. Inspect chip geometry and available controls
    let chip = read_chip_info(&handle)?;
    println!("{}", chip);

    let controls = get_available_controls(&handle);
    // controls is a HashMap<ControlId, ControlValue>; each value carries
    // .min, .max, .step, and .current.

    // 7. Set imaging parameters
    set_param(&handle, ControlId::Gain,     100.0)?;
    set_param(&handle, ControlId::Offset,    10.0)?;
    set_param(&handle, ControlId::Exposure, 10_000_000.0)?;  // 10 s in µs
    set_param(&handle, ControlId::Bits16,    1.0)?;

    // 8. Allocate buffer, expose, retrieve frame
    let buf_size = get_image_buffer_size(&handle) as usize;
    let mut buf = vec![0u8; buf_size];

    exp_single_frame(&handle)?;
    let info = get_single_frame(&handle, &mut buf)?;
    println!("Frame: {}×{} {}bpp", info.width, info.height, info.bpp);

    // … save buf to a FITS/TIFF file, feed it to a pipeline, etc.

    // 9. Close the camera explicitly (also happens automatically on drop)
    close_camera(handle)?;

    // 10. Release SDK resources
    release_resources()?;

    Ok(())
}
```

## API overview

| Function | Description |
|---|---|
| `init_resources` / `release_resources` | SDK lifecycle |
| `get_num_of_connected_cameras` | Scan USB bus |
| `get_camera_id(idx)` | Camera string identifier |
| `open_camera(id)` → `CameraHandle` | Open; handle closes camera on drop |
| `init_camera(handle)` | Upload firmware and apply defaults |
| `set_stream_mode` / `set_read_mode` | Configure before `init_camera` |
| `get_available_controls` | `HashMap<ControlId, ControlValue>` for the connected model |
| `is_control_available` / `set_param` | Query / set any `ControlId` |
| `read_chip_info` | Physical dimensions, resolution, bit depth |
| `read_camera_fw` / `read_sdk_version` | Version info |
| `exp_single_frame` + `get_single_frame` | Capture one frame into a `&mut [u8]` |
| `get_image_buffer_size` | Required buffer size in bytes |

## License

GPL-3.0-only — see [LICENSE](../LICENSE).
