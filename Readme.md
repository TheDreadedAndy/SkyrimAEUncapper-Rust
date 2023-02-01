# Previous Versions

The orignal, C++, implementation of this mod can be found here:
https://github.com/TheDreadedAndy/SkyrimAEUncapper

# Building

1) Install visual studio 2019.
2) Use VS2019 installer to install dependencies.
    * Windows SDK.
    * clang++.
    * MSBuild.
3) Install SKSE64 src as skse64\_src to the workspace directory.
    * Note that, due to limitations with bindgen, the version constants in skse64/src/version.rs must be updated manually.
4) Configure SKSE64 (both release and debug profiles).
    * Retarget to latest version (DO THIS FIRST).
    * Change all compile types to static libs.
    * Add common include path to common project.
    * Disable all post-build actions.
    * Change the C++ runtime library to the DLL version.
    * Disable link-time code generation (else linker errors).
    * Disable whole-program optimization (as above).
    * Test build. Should succeed.
5) Install Rust using rust-up.
6) define LIBCLANG\_PATH environment variable.
7) Update path to MSBuild.exe in skse64/build.rs (if different).
8) Run cargo build.
