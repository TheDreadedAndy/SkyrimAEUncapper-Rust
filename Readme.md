# Previous Versions

[My original, C++, implementation of this mod (versions 1.1.0 and below)](https://github.com/TheDreadedAndy/SkyrimAEUncapper)

[Vadfromnu's SE/AE update to Kassents original mod](https://www.nexusmods.com/skyrimspecialedition/mods/46536?tab=files)

[Kassent's original SE mod](https://github.com/kassent/SkyrimUncapper)

[Elys' LE uncapper, which Kassent's mod is based on](https://www.nexusmods.com/skyrim/mods/1175/)

# Building

1) Install visual studio 2019.
2) Use VS2019 installer to install dependencies.
    * Windows SDK.
    * clang++.
3) Install SKSE64 src as skse64\_src to the workspace directory.
    * Note that, due to limitations with bindgen, the version constants in skse64/src/version.rs must be updated manually.
4) Install Rust using rust-up.
5) define LIBCLANG\_PATH environment variable.
6) Run cargo build.
