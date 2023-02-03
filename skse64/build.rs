//!
//! @file build.rs
//! @brief Generates bindings for the address independence library.
//! @author Andrew Spaulding (Kasplat)
//! @bug No known bugs.
//!

const WRAPPER_FILE: &str = "src/wrapper.cpp";
const STOP_ASM_FILE: &str = "src/stop_plugin.S";

fn main() {
    let vs_profile = vsprofile::VsProfile::get();

    // Mark our header wrapper as a dep.
    println!("cargo:rerun-if-changed={}", STOP_ASM_FILE);
    println!("cargo:rerun-if-changed={}", WRAPPER_FILE);

    // Compile our wrapper files and the parts of skse that we use.
    // No need to build the whole solution, that'd be overkill.
    vs_profile.asm_builder().file(STOP_ASM_FILE).compile("stop_plugin");
    vs_profile.cc_builder()
        .flag("-Isrc/")
        .flag("/FIcommon/IPrefix.h")
        .flag("-Wno-unused-local-typedef")
        .flag("-Wno-unused-parameter")
        .flag("-Wno-parentheses")
        // Note that wrapper.cpp provides IErrors.cpp.
        .file("../skse64_src/common/common/IDataStream.cpp")
        .file("../skse64_src/common/common/IFileStream.cpp")
        .file("../skse64_src/common/common/IDebugLog.cpp")
        .file("../skse64_src/skse64/skse64_common/SafeWrite.cpp")
        .file("../skse64_src/skse64/skse64_common/BranchTrampoline.cpp")
        .file(WRAPPER_FILE)
        .compile("skse64_bind");

    // Link in windows api files that we use.
    println!("cargo:rustc-link-lib=dylib=comdlg32");
    println!("cargo:rustc-link-lib=dylib=Shell32");
    println!("cargo:rustc-link-lib=dylib=Kernel32");
}
