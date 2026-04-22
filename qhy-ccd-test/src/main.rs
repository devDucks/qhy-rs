use libqhy::raw::*;
use log::{debug, error, info};
use rfitsio::hdu::headers::{FITSHeader, FITSValue};
use rfitsio::{FITSFile, HDU};
use std::io::BufRead;

fn probe_camera(idx: u32) -> Result<(), libqhy::raw::QHYError> {
    let id = get_camera_id(idx)?;
    debug!("Found camera {}", id);
    let handle = open_camera(&id)?;
    debug!("Opened camera {} successfully", id);

    match read_camera_fw(&handle) {
        Ok(fw) => debug!("FW version for camera {}: {}", id, fw),
        Err(_) => error!("Couldn't read FW version of camera {}", id),
    }
    match read_sdk_version() {
        Ok(sdk) => debug!("SDK version for camera {}: {}", id, sdk),
        Err(_) => error!("Couldn't read SDK version of camera {}", id),
    }

    match read_chip_info(&handle) {
        Ok(info) => debug!("Chip info => {}", info),
        Err(_) => error!("Couldn't read chip info of camera {}", id),
    }

    debug!(
        "Buffer to fit an image: {} bytes",
        get_image_buffer_size(&handle)
    );

    match get_number_of_read_modes(&handle) {
        Ok(n) => {
            debug!("Number of read modes for camera {}: {}", id, n);
            for mode in 0..n {
                match get_read_mode_name(&handle, mode) {
                    Ok(name) => debug!("  Read mode {}: {}", mode, name),
                    Err(_) => error!("Couldn't read mode name {} of camera {}", mode, id),
                }
            }
        }
        Err(_) => error!("Couldn't get number of read modes of camera {}", id),
    }

    let controls = get_available_controls(&handle);
    debug!("Available controls for camera {}:", id);
    let mut sorted: Vec<_> = controls.iter().collect();
    sorted.sort_by_key(|(k, _)| **k as u32);
    for (ctrl, val) in sorted {
        debug!(
            "  {:?}: current={} min={} max={} step={}",
            ctrl, val.current, val.min, val.max, val.step
        );
    }

    if let Err(e) = acquire_dark_frame(&handle, idx) {
        error!("Dark frame acquisition failed: {}", e);
    }

    match close_camera(handle) {
        Ok(()) => debug!("Camera {} successfully closed", id),
        Err(_) => error!("Couldn't close camera {}", id),
    }
    Ok(())
}

/// Prompt the user to cover the camera, take a 1-second dark frame, verify
/// that at least 90 % of pixels are below 10 % of full scale, then write
/// a conforming FITS file to /tmp/covered.fits.
fn acquire_dark_frame(handle: &CameraHandle, idx: u32) -> Result<(), String> {
    println!(
        "Please cover the camera lens completely, then press Enter to start a 1-second dark exposure..."
    );
    std::io::stdin().lock().lines().next();

    set_stream_mode(&handle, StreamMode::SingleFrame)
        .map_err(|_| "Failed to set single-frame mode")?;
    set_read_mode(&handle, 0).map_err(|_| "Failed to set read mode 0")?;
    init_camera(&handle).map_err(|_| "Failed to initialize camera")?;

    let chip = read_chip_info(&handle).map_err(|_| "Failed to read chip info")?;
    info!(
        "Chip: {}x{} px, {}-bit, pixel size {:.2}x{:.2} um",
        chip.image_width, chip.image_height, chip.bpp, chip.pixel_width, chip.pixel_height
    );

    set_bin_mode(&handle, 1, 1).map_err(|_| "Failed to set 1x1 binning")?;
    set_resolution(&handle, 0, 0, chip.image_width, chip.image_height)
        .map_err(|_| "Failed to set full-frame resolution")?;

    // 1 second expressed in microseconds (SDK unit for CONTROL_EXPOSURE)
    set_param(&handle, ControlId::Exposure, 1_000_000.0)
        .map_err(|_| "Failed to set exposure time")?;

    // Snapshot of controls for metadata (temperature etc.)
    let controls = get_available_controls(&handle);

    let buf_size = get_image_buffer_size(&handle) as usize;
    let mut buf = vec![0u8; buf_size];

    info!("Exposing for 1 second...");
    exp_single_frame(&handle).map_err(|_| "Exposure trigger failed")?;
    let frame = get_single_frame(&handle, &mut buf).map_err(|_| "Failed to read frame data")?;
    info!(
        "Frame acquired: {}x{} @ {}-bit ({} ch)",
        frame.width, frame.height, frame.bpp, frame.channels
    );

    let bytes_per_pixel = (frame.bpp as usize).div_ceil(8);
    let pixel_count = (frame.width * frame.height) as usize;
    let data_len = (pixel_count * bytes_per_pixel).min(buf.len());
    let pixel_bytes = &buf[..data_len];

    // Verify darkness: at least 90 % of pixels must be below 10 % of full scale.
    let dark_threshold: u32 = if frame.bpp <= 8 { 25 } else { 6554 };
    let dark_count: usize = if frame.bpp <= 8 {
        pixel_bytes
            .iter()
            .filter(|&&v| (v as u32) <= dark_threshold)
            .count()
    } else {
        pixel_bytes
            .chunks_exact(2)
            .filter(|c| (u16::from_le_bytes([c[0], c[1]]) as u32) <= dark_threshold)
            .count()
    };
    let dark_frac = dark_count as f64 / pixel_count as f64;
    if dark_frac < 0.90 {
        let _ = release_resources();
        return Err(format!(
            "Only {:.1}% of pixels are dark (expected >90%); ensure the camera is fully covered",
            dark_frac * 100.0
        ));
    }
    info!(
        "{:.1}% of pixels are dark — lens correctly covered",
        dark_frac * 100.0
    );

    // Build the primary FITS HDU
    let mut hdu = HDU::init();
    let bitpix: i64 = if frame.bpp <= 8 { 8 } else { 16 };

    hdu.add_header("SIMPLE", FITSValue::Logical(true));
    hdu.add_header_with_comment("BITPIX", FITSValue::Integer(bitpix), "bits per pixel");
    hdu.add_header_with_comment("NAXIS", FITSValue::Integer(2), "number of array dimensions");
    hdu.add_header_with_comment(
        "NAXIS1",
        FITSValue::Integer(frame.width as i64),
        "image width [px]",
    );
    hdu.add_header_with_comment(
        "NAXIS2",
        FITSValue::Integer(frame.height as i64),
        "image height [px]",
    );
    if frame.bpp > 8 {
        // BZERO / BSCALE convention for unsigned 16-bit stored as signed FITS int
        hdu.add_header_with_comment(
            "BZERO",
            FITSValue::Integer(32768),
            "offset: physical = stored + BZERO",
        );
        hdu.add_header_with_comment("BSCALE", FITSValue::Integer(1), "data scaling factor");
    }
    hdu.add_header_with_comment("EXPTIME", FITSValue::Float(1.0), "[s] total exposure time");
    let id = get_camera_id(idx).map_err(|_| "Failed to get camera id")?;
    hdu.add_header_with_comment(
        "INSTRUME",
        FITSValue::Text(id.chars().take(68).collect()),
        "camera identifier",
    );
    hdu.add_header_with_comment("IMAGETYP", FITSValue::Text("DARK".into()), "frame type");
    hdu.add_header_with_comment(
        "XPIXSZ",
        FITSValue::Float(chip.pixel_width),
        "[um] pixel pitch X",
    );
    hdu.add_header_with_comment(
        "YPIXSZ",
        FITSValue::Float(chip.pixel_height),
        "[um] pixel pitch Y",
    );
    hdu.add_header_with_comment("XBINNING", FITSValue::Integer(1), "horizontal binning");
    hdu.add_header_with_comment("YBINNING", FITSValue::Integer(1), "vertical binning");
    if let Some(t) = controls.get(&ControlId::CurTemp) {
        hdu.add_header_with_comment(
            "CCD-TEMP",
            FITSValue::Float(t.current),
            "[C] sensor temperature",
        );
    }
    hdu.add_comment("Dark frame acquired by qhy-ccd-test (qhy-rs)");
    hdu.headers.push(FITSHeader::end_hdu());

    // Pixel data — 16-bit cameras need the BZERO offset applied before storage
    if frame.bpp <= 8 {
        hdu.data.add_u8(&pixel_bytes[..pixel_count]);
    } else {
        let signed_pixels: Vec<i16> = pixel_bytes
            .chunks_exact(2)
            .map(|c| (u16::from_le_bytes([c[0], c[1]]) as i32 - 32768) as i16)
            .collect();
        hdu.data.add_i16(&signed_pixels);
    }

    let mut fits = FITSFile::new();
    fits.add_hdu(hdu);
    fits.write_to_file("/tmp/covered.fits")
        .map_err(|e| format!("Failed to write FITS file: {}", e))?;

    println!("Dark frame saved to /tmp/covered.fits");
    let _ = release_resources();
    Ok(())
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    let _ = init_resources();
    debug!("Initializing QHY resources");
    let cam_num = get_num_of_connected_cameras();
    debug!("Found {} cameras", cam_num);

    for idx in 0..cam_num {
        if let Err(_) = probe_camera(idx) {
            error!("Failed to probe camera {}", idx);
        }
    }

    if release_resources().is_ok() {
        debug!("All QHY resources released");
    } else {
        error!("Unable to release QHY resources, unplug them");
    }
}
