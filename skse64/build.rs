//!
//! @file build.rs
//! @brief Generates bindings for the address independence library.
//! @author Andrew Spaulding (aspauldi)
//! @bug No known bugs.
//!

use std::env;
use std::path::PathBuf;
use std::process::Command;

const WRAPPER_FILE: &str = "src/wrapper.cpp";
const BINDGEN_FILE: &str = "bindgen_wrapper.h";
const MSBUILD: &str = "C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\Community\\MSBuild\\Current\\Bin\\MSBuild.exe";

fn main() {
    // Mark our header wrapper as a dep.
    println!("cargo:rerun-if-changed={}", BINDGEN_FILE);
    println!("cargo:rerun-if-changed={}", WRAPPER_FILE);

    // Compile our wrapper.
    cc::Build::new()
        .cpp(true)
        .file(WRAPPER_FILE)
        .cpp_link_stdlib("stdc++")
        .flag("-Isrc/")
        .flag("-I../skse64_src/common/")
        .flag("-I../skse64_src/skse64/skse64_common/")
        .compile("libwrapper.a");

    // Generate the bindings.
    let bindings = bindgen::Builder::default()
        .header(BINDGEN_FILE)
        .use_core()
        .clang_arg("-xc++")
        .clang_arg("-std=c++17")
        .clang_arg("-I../skse64_src/common/")
        .clang_arg("-I../skse64_src/skse64/")
        .clang_arg("-I../skse64_src/skse64/skse64/")
        .clang_arg("-I../skse64_src/skse64/skse64_common/")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate().unwrap();

    let binding_file = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    bindings.write_to_file(binding_file).unwrap();

    // Build skse64.
    Command::new(MSBUILD).args(
        &["../skse64_src/skse64/skse64.sln", "-t:Build", "-p:Configuration=Debug"]
    ).status().unwrap();

    // Add directories to search for libs in.
    println!("cargo:rustc-link-search=skse64_src/skse64/x64/Debug/");
    println!("cargo:rustc-link-search=skse64_src/skse64/x64_v142/Debug/");

    // Link in skse64.
    println!("cargo:rustc-link-lib=static=skse64_1_6_323");
    println!("cargo:rustc-link-lib=static=skse64_common");
    println!("cargo:rustc-link-lib=static=skse64_loader");
    println!("cargo:rustc-link-lib=static=skse64_loader_common");
    println!("cargo:rustc-link-lib=static=skse64_steam_loader");
    println!("cargo:rustc-link-lib=static=common_vc14");
}
