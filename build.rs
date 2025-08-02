// Made by deepseek to save me from whatever this stupid syntax is

fn main() {
    // Build C libraries as static
    let dst = cmake::Config::new(".")
        .define("BUILD_SHARED_LIBS", "OFF") // Force static build
        .build();

    // Set library search path
    let lib_dir = dst.join("build/lib");
    println!("cargo:rustc-link-search={}", lib_dir.display());

    // Link static libraries
    println!("cargo:rustc-link-lib=static=diskman");
    println!("cargo:rustc-link-lib=static=tui");

    // Find and link ncurses dynamically (system library)
    pkg_config::Config::new()
        .atleast_version("6") // Minimum ncurses version
        .probe("ncurses")
        .expect("ncurses library not found");

    // Re-run build if C sources change
    println!("cargo:rerun-if-changed=c_src/tui/tui.c");
    println!("cargo:rerun-if-changed=c_src/tui/tui.h");
    println!("cargo:rerun-if-changed=c_src/diskman/diskman.c");
    println!("cargo:rerun-if-changed=c_src/diskman/diskman.h");
}
