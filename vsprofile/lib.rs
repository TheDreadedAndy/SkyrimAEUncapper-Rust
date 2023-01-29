//!
//! @file lib.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Helps configure builds compatible with VS2019.
//! @bug No known bugs.
//!

/// @brief The supported VS profiles.
#[derive(Copy, Clone)]
pub enum VsProfile {
    Release,
    Debug
}

impl VsProfile {
    /// @brief Gets the VsProfile which should be used with the running cargo profile.
    pub fn get() -> Self {
        match std::env::var("PROFILE").as_ref().map(|s| s.as_str()) {
            Ok("debug") => Self::Debug,
            Ok("release") => Self::Release,
            _ => panic!("{}", "Unknown profile")
        }
    }

    /// @brief Gets the profile name associated with this profile.
    pub fn name(
        self
    ) -> &'static str {
        match self {
            Self::Debug => "Debug",
            Self::Release => "Release"
        }
    }

    ///
    /// Configures a asm builder for compatibility with the VS profile.
    ///
    pub fn asm_builder(
        self
    ) -> cc::Build {
        Self::clang_builder("clang.exe")
    }

    ///
    /// Configures a cc builder for compatibility with the VS profile.
    ///
    /// Additionally prepares build settings which vs2019 would otherwise have.
    ///
    pub fn cc_builder(
        self
    ) -> cc::Build {
        let mut builder = Self::clang_builder("clang-cl.exe");
        builder.cpp(true)
            .flag("/EHs")
            .static_crt(false)
            .include("../skse64/inc/")
            .include("../skse64_src/common/")
            .include("../skse64_src/skse64/")
            .include("../skse64_src/skse64/skse64_common/");

        match self {
            Self::Debug => {
                builder.define("_DEBUG", None)
                    .define("_ITERATOR_DEBUG_LEVEL", Some("2"))
                    .opt_level_str("1")
                    .cpp_link_stdlib("msvcrtd"); // Debug dynamic windows stdc++ lib.
            }
            Self::Release => {
                builder.opt_level_str("z")
                    .cpp_link_stdlib("msvcrt"); // Release dynamic windows stdc++ lib.
            }
        }

        return builder;
    }

    /// Creates a builder with the given clang compiler.
    fn clang_builder(
        exe: &str
    ) -> cc::Build {
        let mut builder = cc::Build::new();
        let clang_path = std::env::var("LIBCLANG_PATH").unwrap();
        let clang = std::path::PathBuf::from(clang_path).join(exe);
        builder.compiler(clang);
        return builder;
    }
}
