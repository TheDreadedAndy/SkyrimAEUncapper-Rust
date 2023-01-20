//!
//! @file build.rs
//! @brief Builds the wrapper file for the address independence library.
//! @author Andrew Spaulding (aspauldi)
//! @bug No known bugs.
//!

const WRAPPER_FILE: &str = "src/wrapper.cpp";

fn main() {
    // Mark our header wrapper as a dep.
    println!("cargo:rerun-if-changed={}", WRAPPER_FILE);

    // Compile our wrapper.
    cc::Build::new()
        .cpp(true)
        .file(WRAPPER_FILE)
        .flag("-Isrc/")
        .flag("-I../skse64_src/common/")
        .compile("libwrapper.a");
}
