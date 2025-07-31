// Thank you github copilot for saving me from this sketchy syntax!
use std::fs;

fn main() {
    // Build C library with CMake
    let dst = cmake::Config::new(".").build();

    // Get the output directory for the built library
    let libtui = dst.join("build/lib/libtui.so");
    let libdisk = dst.join("build/lib/libdiskman.so");

    // Get target directory (debug/release)
    let profile = std::env::var("PROFILE").unwrap();
    let target_dir = format!(
        "{}/target/{}",
        std::env::var("CARGO_MANIFEST_DIR").unwrap(),
        profile
    );
    let libtui_dest = format!("{}/libtui.so", target_dir);
    let libdisk_dest = format!("{}/libdiskman.so", target_dir);

    // Copy the library to the target directory
    fs::copy(&libtui, &libtui_dest).expect("Failed to copy library tui");
    fs::copy(&libdisk, &libdisk_dest).expect("Failed to copy library diskman");

    // Set up library search paths
    println!(
        "cargo:rustc-link-search={}",
        dst.join("build/lib").display()
    );
    println!("cargo:rustc-link-search={}", target_dir);

    // Configure rpath to look in the same directory as the executable
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");

    println!("cargo:rustc-link-lib=dylib=tui");
    println!("cargo:rustc-link-lib=dylib=diskman");

    // Re-run build if C files change
    println!("cargo:rerun-if-changed=c_src/tui/tui.c");
    println!("cargo:rerun-if-changed=c_src/tui/tui.h");
    println!("cargo:rerun-if-changed=c_src/diskman/diskman.c");
    println!("cargo:rerun-if-changed=c_src/diskman/diskman.h");
}
