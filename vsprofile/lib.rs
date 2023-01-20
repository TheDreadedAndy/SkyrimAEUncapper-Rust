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

    /// @brief Configures a cc builder for compatibility with the VS profile.
    pub fn config_builder(
        self,
        builder: &mut cc::Build
    ) {
        match self {
            Self::Debug => {
                builder.flag("-D_DEBUG").flag("-D_ITERATOR_DEBUG_LEVEL=2");
                builder.cpp_link_stdlib("msvcrtd"); // Debug dynamic windows stdc++ lib.
            }
            Self::Release => {
                builder.cpp_link_stdlib("msvcrt"); // Release dynamic windows stdc++ lib.
            }
        }
    }
}
