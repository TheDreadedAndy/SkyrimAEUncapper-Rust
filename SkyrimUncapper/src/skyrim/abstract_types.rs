//!
//! @file abstract_types.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Defines abstract types from skyrim AE.
//! @bug No known bugs.
//!

use ctypes::abstract_type;

abstract_type! {
    /// @brief The player actor class.
    pub type PlayerActor;

    /// @brief The class which manages skills/attributes for an actor.
    pub type ActorValueOwner;

    /// @brief Player skill game class.
    pub type PlayerSkills;
}
