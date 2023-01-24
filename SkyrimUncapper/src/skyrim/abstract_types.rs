//!
//! @file abstract_types.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Defines abstract types from skyrim AE.
//! @bug No known bugs.
//!

use ctypes::abstract_type;
use skse64::version::{RUNTIME_VERSION_1_6_317, RUNTIME_VERSION_1_6_629};
use skse64::errors::skse_assert;

abstract_type! {
    /// The player actor class.
    pub type PlayerActor;

    /// The class which manages skills/attributes for an actor.
    pub type ActorValueOwner;

    /// Player skill game class.
    pub type PlayerSkills;

    /// Contains configuration settings exposed by the game engine.
    pub type SettingCollectionMap;
}

///
/// Gets the actor value owner of the given player actor.
///
/// The given pointer must point to a valid player actor structure.
///
pub unsafe fn get_player_avo(
    player: *mut PlayerActor
) -> *mut ActorValueOwner {
    skse_assert!(!player.is_null());

    let version = skse64::version::current_runtime();
    skse_assert!(version >= RUNTIME_VERSION_1_6_317); // AE.
    let offset: usize = if version >= RUNTIME_VERSION_1_6_629 { 0xb8 } else { 0xb0 };
    player.cast::<u8>().add(offset).cast::<ActorValueOwner>()
}
