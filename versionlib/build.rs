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
    vsprofile::VsProfile::get().cc_builder().file(WRAPPER_FILE).flag("-Isrc/").compile("wrapper");
}
