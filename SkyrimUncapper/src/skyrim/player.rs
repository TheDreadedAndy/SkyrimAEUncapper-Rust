//!
//! @file abstract_types.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Defines abstract player structures from skyrim AE.
//! @bug No known bugs.
//!

use skse64::version::{RUNTIME_VERSION_1_6_317, RUNTIME_VERSION_1_6_629};

skse64::util::abstract_type! {
    /// The player actor class.
    pub type PlayerCharacter;

    /// The class which manages skills/attributes for an actor.
    pub type ActorValueOwner;

    /// Player skill game class.
    pub type PlayerSkills;
}

impl PlayerCharacter {
    /// Gets the actor value owner for the player actor.
    pub fn get_avo(
        &self
    ) -> *mut ActorValueOwner {
        let version = skse64::version::current_runtime();
        assert!(version >= RUNTIME_VERSION_1_6_317); // AE.

        let offset: usize = if version >= RUNTIME_VERSION_1_6_629 { 0xb8 } else { 0xb0 };
        let player = self as *const Self as *mut Self;
        unsafe {
            // SAFETY: We have ensured that we are using the correct offset for our game version.
            player.cast::<u8>().add(offset).cast::<ActorValueOwner>()
        }
    }
}
