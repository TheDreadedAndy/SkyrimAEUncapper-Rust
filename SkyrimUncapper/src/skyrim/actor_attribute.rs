//!
//! @file actor_attribute.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief The ActorAttribute game enumeration, and its methods.
//! @bug No known bugs.
//!

use std::ffi::c_int;

use skse64::errors::skse_assert;

/// @brief The number of skills the player has.
pub const SKILL_COUNT: usize = 18;

/// @brief The offset in the attribute enum to the start of the skill block.
const SKILL_OFFSET: usize = 6;

/// @brief Encodes the actor attribute enum, as defined by the game.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ActorAttribute {
    /* 0x0-0x5 unknown */
    OneHanded = 0x6,
    TwoHanded,
    Marksman,
    Block,
    Smithing,
    HeavyArmor,
    LightArmor,
    Pickpocket,
    LockPicking,
    Sneak,
    Alchemy,
    Speechcraft,
    Alteration,
    Conjuration,
    Destruction,
    Illusion,
    Restoration,
    Enchanting,
    Health,
    Magicka,
    Stamina,
    /* 0x1b-0x1f unknown */
    CarryWeight = 0x20
}

impl ActorAttribute {
    /// @brief Converts a c_int into an ActorAttribute, if it has a known value.
    pub fn from_raw(
        attr: c_int
    ) -> Result<Self, ()> {
        if (((ActorAttribute::OneHanded as c_int) <= attr) &&
                (attr <= (ActorAttribute::Stamina as c_int))) ||
                (attr == (ActorAttribute::CarryWeight as c_int)) {
            Ok(unsafe {
                // SAFETY: We confirmed this is a valid actor attribute.
                std::mem::transmute::<c_int, ActorAttribute>(attr)
            })
        } else {
            Err(())
        }
    }

    /// @brief Checks if the invoking attribute is a skill.
    pub fn is_skill(
        self
    ) -> bool {
        return ((ActorAttribute::OneHanded as usize) <= (self as usize)) &&
            ((self as usize) <= (ActorAttribute::Enchanting as usize));
    }

    ///
    /// @brief Converts the attribute into a skill slot.
    ///
    /// The invoking attribute must be a skill.
    ///
    pub fn skill_slot(
        self
    ) -> usize {
        skse_assert!(self.is_skill());
        (self as usize) - SKILL_OFFSET
    }

    /// @brief Gets the string name of the actor attribute.
    pub fn name(
        self
    ) -> &'static str {
        match self {
            ActorAttribute::OneHanded => "OneHanded",
            ActorAttribute::TwoHanded => "TwoHanded",
            ActorAttribute::Marksman => "Marksman",
            ActorAttribute::Block => "Block",
            ActorAttribute::Smithing => "Smithing",
            ActorAttribute::HeavyArmor => "HeavyArmor",
            ActorAttribute::LightArmor => "LightArmor",
            ActorAttribute::Pickpocket => "Pickpocket",
            ActorAttribute::LockPicking => "LockPicking",
            ActorAttribute::Sneak => "Sneak",
            ActorAttribute::Alchemy => "Alchemy",
            ActorAttribute::Speechcraft => "SpeechCraft", // Legacy: case.
            ActorAttribute::Alteration => "Alteration",
            ActorAttribute::Conjuration => "Conjuration",
            ActorAttribute::Destruction => "Destruction",
            ActorAttribute::Illusion => "Illusion",
            ActorAttribute::Restoration => "Restoration",
            ActorAttribute::Enchanting => "Enchanting",
            ActorAttribute::Health => "Health",
            ActorAttribute::Magicka => "Magicka",
            ActorAttribute::Stamina => "Stamina",
            ActorAttribute::CarryWeight => "CarryWeight"
        }
    }
}
