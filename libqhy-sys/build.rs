extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn vendored_lib_dir() -> PathBuf {
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let arch = match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        arch => panic!("unsupported architecture: {}", arch),
    };
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("lib")
        .join(os)
        .join(arch)
}

fn camera_bindings() {
    if pkg_config::probe_library("qhyccd").is_err() {
        let lib_dir = env::var("QHYCCD_LIB_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| vendored_lib_dir());
        println!(
            "cargo:rustc-link-search={}",
            lib_dir.canonicalize().expect("QHYCCD_LIB_DIR not found").display()
        );
        println!("cargo:rustc-link-lib=qhyccd");
    }

    println!("cargo:rerun-if-changed=./include/wrapper.h");
    println!("cargo:rerun-if-changed=./include/config.h");
    println!("cargo:rerun-if-changed=./include/qhyccdcamdef.h");
    println!("cargo:rerun-if-changed=./include/qhyccderr.h");
    println!("cargo:rerun-if-changed=./include/qhyccd.h");
    println!("cargo:rerun-if-changed=./include/qhyccdstruct.h");

    let bindings = bindgen::Builder::default()
        .header("./include/wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("cam_bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    camera_bindings();
}
