//!
//! @file abstract_types.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Defines abstract player structures from skyrim AE.
//! @bug No known bugs.
//!

use core::cell::Cell;

use skse64::version::RUNTIME_VERSION_1_6_629;

keywords::abstract_type! {
    /// The player actor class.
    pub type PlayerCharacter;

    /// The class which manages skills/attributes for an actor.
    pub type ActorValueOwner;
}

impl PlayerCharacter {
    /// Gets the actor value owner for the player actor.
    pub fn get_avo(
        &self
    ) -> *mut ActorValueOwner {
        unsafe {
            // SAFETY: These offsets have been verified to be correct.
            self.version_offset(0xb8, 0xb0)
        }
    }

    /// Gets a reference to the players perk pool.
    pub fn get_perk_pool(
        &self
    ) -> &Cell<u8> {
        unsafe {
            // SAFETY: These offsets have been verified to be correct. Cell is transparent, so we
            //         can use it here as a safe wrapper around a variable that we don't have
            //         exclusive access to.
            Cell::from_mut(self.version_offset::<u8>(0xb09, 0xb01).as_mut().unwrap())
        }
    }

    /// Gets a version dependent offset in the player structure.
    unsafe fn version_offset<T>(
        &self,
        current: usize,
        compat: usize
    ) -> *mut T {
        let version = skse64::version::current_runtime();
        let offset: usize = if version >= RUNTIME_VERSION_1_6_629 { current } else { compat };
        let player = self as *const Self as *mut Self;
        player.cast::<u8>().add(offset).cast()
    }
}
