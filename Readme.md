# Building

1) Install visual studio 2019.
2) Use VS2019 installer to install dependencies.
    a) Windows SDK.
    b) clang++.
    c) MSBuild.
3) Configure SKSE64 (both release and debug profiles).
    a) Retarget to latest version (DO THIS FIRST).
    b) Change all compile types to static libs.
    c) Add common include path to common project.
    d) Disable all post-build actions.
    e) Change the C++ runtime library to the DLL version.
    f) Test build. Should succeed.
4) Install Rust using rust-up.
5) define LIBCLANG\_PATH environment variable.
6) Update path to MSBulid.exe in skse64/build.rs (if different).
7) Run cargo build.
