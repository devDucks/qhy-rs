extern crate bindgen;

use std::env;
use std::path::PathBuf;

//config.h  qhyccdcamdef.h  qhyccderr.h  qhyccd.h  qhyccdstruct.h
fn camera_bindings() {
    let path = std::fs::canonicalize("../vendored/camera/linux/x64");
    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search={}", path.unwrap().display());

    // Tell cargo to tell rustc to link the SDK
    println!("cargo:rustc-link-lib=qhyccd");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=./include/wrapper.h");
    println!("cargo:rerun-if-changed=./include/config.h");
    println!("cargo:rerun-if-changed=./include/qhyccdcamdef.h");
    println!("cargo:rerun-if-changed=./include/qhyccderr.h");
    println!("cargo:rerun-if-changed=./include/qhyccd.h");
    println!("cargo:rerun-if-changed=./include/qhyccdstruct.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // wrapper.h includes all SDK headers with __CPP_MODE__ forced to 0
        // (config.h defines it as 1, which would pull in C++-only headers)
        // and includes stdbool.h so `bool` is recognized in C mode.
        .header("./include/wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("cam_bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    camera_bindings();
}
