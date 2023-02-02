//!
//! @file actor_attribute.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief The ActorAttribute game enumeration, and its methods.
//! @bug No known bugs.
//!

use std::ffi::c_int;

macro_rules! attr_name {
    ( $pre:literal, $attr:expr ) => {
        match $attr {
            ActorAttribute::OneHanded   => ::std::concat!($pre, "OneHanded"),
            ActorAttribute::TwoHanded   => ::std::concat!($pre, "TwoHanded"),
            ActorAttribute::Marksman    => ::std::concat!($pre, "Marksman"),
            ActorAttribute::Block       => ::std::concat!($pre, "Block"),
            ActorAttribute::Smithing    => ::std::concat!($pre, "Smithing"),
            ActorAttribute::HeavyArmor  => ::std::concat!($pre, "HeavyArmor"),
            ActorAttribute::LightArmor  => ::std::concat!($pre, "LightArmor"),
            ActorAttribute::Pickpocket  => ::std::concat!($pre, "Pickpocket"),
            ActorAttribute::LockPicking => ::std::concat!($pre, "LockPicking"),
            ActorAttribute::Sneak       => ::std::concat!($pre, "Sneak"),
            ActorAttribute::Alchemy     => ::std::concat!($pre, "Alchemy"),
            ActorAttribute::Speechcraft => ::std::concat!($pre, "SpeechCraft"), // Legacy: case.
            ActorAttribute::Alteration  => ::std::concat!($pre, "Alteration"),
            ActorAttribute::Conjuration => ::std::concat!($pre, "Conjuration"),
            ActorAttribute::Destruction => ::std::concat!($pre, "Destruction"),
            ActorAttribute::Illusion    => ::std::concat!($pre, "Illusion"),
            ActorAttribute::Restoration => ::std::concat!($pre, "Restoration"),
            ActorAttribute::Enchanting  => ::std::concat!($pre, "Enchanting"),
            ActorAttribute::Health      => ::std::concat!($pre, "Health"),
            ActorAttribute::Magicka     => ::std::concat!($pre, "Magicka"),
            ActorAttribute::Stamina     => ::std::concat!($pre, "Stamina"),
            ActorAttribute::CarryWeight => ::std::concat!($pre, "CarryWeight")
        }
    };
}

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

/// @brief Iterates over the skills of an actor.
pub struct SkillIterator(Option<ActorAttribute>);

// Private module for sealed hungarian type trait.
mod _private {
    pub trait Sealed {}
    impl Sealed for f32 {}
    impl Sealed for u32 {}
}

pub trait HungarianAttribute: _private::Sealed + Copy {
    fn hungarian_attr(attr: ActorAttribute) -> &'static str;
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
        assert!(self.is_skill());
        (self as usize) - SKILL_OFFSET
    }

    /// @brief Gets the string name of the actor attribute.
    pub fn name(
        self
    ) -> &'static str {
        attr_name!("", self)
    }
}

impl HungarianAttribute for f32 {
    fn hungarian_attr(
        attr: ActorAttribute
    ) -> &'static str {
        attr_name!("f", attr)
    }
}

impl HungarianAttribute for u32 {
    fn hungarian_attr(
        attr: ActorAttribute
    ) -> &'static str {
        attr_name!("i", attr)
    }
}

impl SkillIterator {
    /// @brief Creates a new skill iterator.
    pub fn new() -> Self {
        Self(Some(ActorAttribute::OneHanded))
    }
}

impl Iterator for SkillIterator {
    type Item = ActorAttribute;

    fn next(
        &mut self
    ) -> Option<Self::Item> {
        let ret = self.0.clone();
        self.0 = match ret {
            Some(ActorAttribute::OneHanded) => Some(ActorAttribute::TwoHanded),
            Some(ActorAttribute::TwoHanded) => Some(ActorAttribute::Marksman),
            Some(ActorAttribute::Marksman) => Some(ActorAttribute::Block),
            Some(ActorAttribute::Block) => Some(ActorAttribute::Smithing),
            Some(ActorAttribute::Smithing) => Some(ActorAttribute::HeavyArmor),
            Some(ActorAttribute::HeavyArmor) => Some(ActorAttribute::LightArmor),
            Some(ActorAttribute::LightArmor) => Some(ActorAttribute::Pickpocket),
            Some(ActorAttribute::Pickpocket) => Some(ActorAttribute::LockPicking),
            Some(ActorAttribute::LockPicking) => Some(ActorAttribute::Sneak),
            Some(ActorAttribute::Sneak) => Some(ActorAttribute::Alchemy),
            Some(ActorAttribute::Alchemy) => Some(ActorAttribute::Speechcraft),
            Some(ActorAttribute::Speechcraft) => Some(ActorAttribute::Alteration),
            Some(ActorAttribute::Alteration) => Some(ActorAttribute::Conjuration),
            Some(ActorAttribute::Conjuration) => Some(ActorAttribute::Destruction),
            Some(ActorAttribute::Destruction) => Some(ActorAttribute::Illusion),
            Some(ActorAttribute::Illusion) => Some(ActorAttribute::Restoration),
            Some(ActorAttribute::Restoration) => Some(ActorAttribute::Enchanting),
            Some(ActorAttribute::Enchanting) => None,
            None => None,
            _ => unreachable!()
        };
        return ret;
    }
}
