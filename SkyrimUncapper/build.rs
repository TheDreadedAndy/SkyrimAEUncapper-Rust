//!
//! @file build.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Builds/links the ASM and resource file for the uncapper.
//! @bug No known bugs.
//!

const RC_AUTHOR: &str = "Kasplat";
const RC_NAME: &str = "Skyrim Uncapper AE";
const RC_VERSION: &str = "2.0.1.0";
const RC_FILE: &str = "SkyrimUncapper.dll";

fn main() {
    // Build C++ exception nets.
    cc::Build::new().cpp(true).file("src/skyrim/native_wrappers.cpp").compile("nets");

    // Embed resource information.
    let mut res = winres::WindowsResource::new();
    res.set("CompanyName", RC_AUTHOR);
    res.set("FileDescription", RC_NAME);
    res.set("FileVersion", RC_VERSION);
    res.set("InternalName", RC_FILE);
    res.set("LegalCopyright", "Copyright (C) 2023");
    res.set("OriginalFilename", RC_FILE);
    res.set("ProductName", RC_NAME);
    res.set("ProductVersion", RC_VERSION);
    res.compile().unwrap();
}
