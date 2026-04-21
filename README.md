# qhy-rs
Lightspeed compliant Rust drivers for QHY astronomical equipment

## Prerequisites

This crate wraps the [QHY CCD SDK](https://www.qhyccd.com/html/prepub/log_en.html#!log_en.md), a proprietary prebuilt library.

**If you cloned this repository**, no installation is needed — the build script falls back to the vendored SDK copy included under `vendored/`.

**If you are pulling this crate from crates.io**, you must install the SDK first. The build script looks for it in the following order:

1. Via `pkg-config`
2. Via the `QHYCCD_LIB_DIR` environment variable pointing to the directory containing `libqhyccd.so`

### Arch Linux

Install [`indi-3rdparty-libs`](https://aur.archlinux.org/packages/indi-3rdparty-libs) from the AUR:

```sh
paru -S indi-3rdparty-libs
```

### Ubuntu / Debian

Install via the [INDI third-party PPA](https://launchpad.net/~mutlaqja/+archive/ubuntu/ppa):

```sh
sudo add-apt-repository ppa:mutlaqja/ppa
sudo apt install libqhy
```

### Other

Download the SDK from the [QHY website](https://www.qhyccd.com/html/prepub/log_en.html#!log_en.md) and follow their installation instructions, or set `QHYCCD_LIB_DIR` to the directory containing `libqhyccd.so`:

```sh
QHYCCD_LIB_DIR=/path/to/sdk/lib cargo build
```
