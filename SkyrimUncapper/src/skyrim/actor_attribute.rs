//!
//! @file actor_attribute.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief The ActorAttribute game enumeration, and its methods.
//! @bug No known bugs.
//!

/// @brief Encodes the actor attribute enum, as defined by the game.
#[repr(C)]
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
