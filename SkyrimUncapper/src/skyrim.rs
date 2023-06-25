//!
//! @file skyrim.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Defines types and functions internal to Skyrim SE/AE.
//! @bug No known bugs.
//!
//! As this mod is only really concerned with changing the players leveling and stats, this file
//! exposes only the game objects which manage this data. In particular, the player object, actor
//! value object, actor attribute enumeration, and various game constants related to leveling and
//! the player are exposed within this file.
//!
//! The player object provides a number of static methods to access various fields within the
//! player structure. When necessary, these fields are accessed by manually offsetting the player
//! pointer based on the game version, as the location of many fields within the player structures
//! changed in AE 1.6.629.
//!
//! The actor value owner object is simply an abstract C type that can be passed to other
//! functions. It has no methods and cannot be directly created in rust code.
//!
//! The actor attribute enumeration only defines the values which need to be directly accessed by
//! this mod. As such, it is not, in general, safe to convert integers that encode event valid
//! values of the structure to our internal representation unless it is known that those integers
//! will always be of values that we can encode, as in many of the players leveling functions.
//! Otherwise, it is necessary to treat the actor attribute as an opaque integer.
//!
//! Note that the game functions this file hooks into are wrapped by exception handlers defined in
//! C++ code to ensure that they cannot unwrap into this mods code and cause U.B. This extra layer
//! of goop will be unnecessary once the "system-unwind" ABI becomes stablized.

use core::cell::Cell;
use core::ffi::c_int;

use skyrim_patcher::{GameRef, Descriptor, GameLocation};
use skse64::version::RUNTIME_VERSION_1_6_629;

use crate::settings::SkillMult;
use crate::settings::SETTINGS;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Game object definitions
////////////////////////////////////////////////////////////////////////////////////////////////////

/// The number of skills the player has.
pub const SKILL_COUNT: usize = 18;

/// The offset in the attribute enum to the start of the skill block.
const SKILL_OFFSET: usize = 6;

core_util::abstract_type! {
    /// The player actor class.
    pub type PlayerCharacter;

    /// The class which manages skills/attributes for an actor.
    pub type ActorValueOwner;
}

///
/// Encodes the actor attribute enum, as defined by the game.
///
/// This enum actually has *many* more values (163), but I refuse to transcribe them all.
/// The full list is here: https://en.uesp.net/wiki/Skyrim_Mod:Actor_Value_Indices
///
#[repr(C)]
#[allow(dead_code)] // Transmutes don't count as usage.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ActorAttribute {
    /* 0x0-0x5 ignored */
    OneHanded = SKILL_OFFSET as isize,
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
    /* 0x1b-0x1f ignored */
    CarryWeight = 0x20
}

/// Iterates over the skills of an actor.
pub struct SkillIterator(Option<ActorAttribute>);

////////////////////////////////////////////////////////////////////////////////////////////////////
// Player object wrapper
////////////////////////////////////////////////////////////////////////////////////////////////////

impl PlayerCharacter {
    /// Gets the current level of the player.
    pub fn get_level() -> u32 {
        unsafe { get_level_net(*(PLAYER_OBJECT.get())) as u32 }
    }

    /// Gets a reference to the players perk pool.
    pub fn get_perk_pool() -> &'static Cell<u8> {
        // SAFETY: These offsets have been verified to be correct. Cell is transparent, so we
        //         can use it here as a safe wrapper around a variable that we don't have
        //         exclusive access to.
        unsafe { Cell::from_mut(Self::version_offset::<u8>(0xb09, 0xb01).as_mut().unwrap()) }
    }

    /// Gets the actor value owner for the player actor.
    ///
    /// Called from ASM code, so we must mark it as extern "system".
    pub extern "system" fn get_avo() -> *mut ActorValueOwner {
        // SAFETY: These offsets have been verified to be correct.
        unsafe { Self::version_offset(0xb8, 0xb0) }
    }

    /// Gets the base value of the given attribute.
    pub fn get_base(
        attr: ActorAttribute
    ) -> f32 {
        unsafe { Self::get_base_unchecked(attr as c_int) }
    }

    /// Gets the current value of the given attribute.
    pub fn get_current(
        attr: ActorAttribute
    ) -> f32 {
        unsafe { Self::get_current_unchecked(attr as c_int) }
    }

    /// Modifies the base value of the given attribute.
    pub fn mod_base(
        attr: ActorAttribute,
        val: f32
    ) {
        unsafe { player_avo_mod_base_net(Self::get_avo(), attr as c_int, val) }
    }

    /// Modifies the current value of a player attribute.
    pub fn mod_current(
        attr: ActorAttribute,
        val: f32
    ) {
        // No idea what second arg does; just match game calls.
        unsafe { player_avo_mod_current_net(Self::get_avo(), 0, attr as c_int, val) }
    }

    ///
    /// Gets the base value of a player attribute.
    ///
    /// In order to use this function safely, the given attribute must be valid.
    ///
    /// Marked as extern system, since it is called from assembly code.
    ///
    pub unsafe extern "system" fn get_base_unchecked(
        attr: c_int
    ) -> f32 {
        player_avo_get_base_net(Self::get_avo(), attr)
    }

    ///
    /// Gets the base value of a player attribute.
    ///
    /// In order to use this function safely, the given attribute must be valid.
    ///
    /// Marked as extern system, since it is called from assembly code.
    ///
    pub unsafe extern "system" fn get_current_unchecked(
        attr: c_int
    ) -> f32 {
        avo_get_current_unchecked(
            Self::get_avo(),
            attr,
        )
    }

    /// Gets a version dependent offset in the player structure.
    unsafe fn version_offset<T>(
        current: usize,
        compat: usize
    ) -> *mut T {
        // SAFETY: We know the player pointer is valid, as GameRef ensures this.
        let version = skse64::version::current_runtime();
        let offset: usize = if version >= RUNTIME_VERSION_1_6_629 { current } else { compat };
        let player = *(PLAYER_OBJECT.get());
        player.cast::<u8>().add(offset).cast()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Actor attribute convenience methods
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Prints the attribute with the given prefix to its name. Used for printing hungarian type names.
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

// Private module for sealed hungarian type trait.
mod _private {
    pub trait Sealed {}
    impl Sealed for f32 {}
    impl Sealed for u32 {}
    impl Sealed for crate::settings::SkillMult {}
}

pub trait HungarianAttribute: _private::Sealed + Copy {
    fn hungarian_attr(attr: ActorAttribute) -> &'static str;
}

impl ActorAttribute {
    /// Converts a c_int into an ActorAttribute, if it has a known value.
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

    /// Converts a c_int into a skill attribute, if the value is a known skill value.
    pub fn from_raw_skill(
        attr: c_int
    ) -> Result<Self, ()> {
        Self::from_raw(attr).and_then(|a| if a.is_skill() { Ok(a) } else { Err(()) })
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

impl HungarianAttribute for SkillMult {
    fn hungarian_attr(
        attr: ActorAttribute
    ) -> &'static str {
        attr_name!("f", attr)
    }
}

impl SkillIterator {
    pub fn new() -> Self {
        Self(Some(ActorAttribute::OneHanded))
    }
}

impl Iterator for SkillIterator {
    type Item = ActorAttribute;
    fn next(
        &mut self
    ) -> Option<Self::Item> {
        let ret = self.0;
        self.0 = if let Some(attr) = self.0 {
            if attr == ActorAttribute::Enchanting {
                None
            } else {
                // The underlying representation of an actor attribute is a c_int, and we know that the
                // order we want to iterate in is the same as the definition order, so we just
                // increment, as a simplification.
                Some(ActorAttribute::from_raw(attr as c_int + 1).unwrap())
            }
        } else {
            None
        };
        return ret;
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Game object hooks
////////////////////////////////////////////////////////////////////////////////////////////////////
//
// Note that instead of hooking into the game constant management object, we hook into each
// constant we use individually. This saves us a lookup, and means we don't need to import as many
// game structures and functions. We instead import more constants, which are easier to deal with
// anyway.

// A pointer to the global player variable in the skyrim binary.
static PLAYER_OBJECT: GameRef<*mut *mut PlayerCharacter> = GameRef::new();

// Game constants, which are also available through the settings map.
pub static ENCHANTING_SKILL_COST_BASE: GameRef<&'static f32> = GameRef::new();
pub static ENCHANTING_SKILL_COST_SCALE: GameRef<&'static f32> = GameRef::new();
pub static ENCHANTING_COST_EXPONENT: GameRef<&'static f32> = GameRef::new();
pub static ENCHANTING_SKILL_COST_MULT: GameRef<&'static f32> = GameRef::new();
pub static XP_PER_SKILL_RANK: GameRef<&'static f32> = GameRef::new();
pub static LEGENDARY_SKILL_RESET_VALUE: GameRef<&'static f32> = GameRef::new();

// Game functions. These are wrapped by C++ catchers and then safe implementations later.
#[no_mangle]
static get_level_entry: GameRef<fn(*mut PlayerCharacter) -> u16> = GameRef::new();
#[no_mangle]
static player_avo_get_base_entry: GameRef<
    unsafe extern "system" fn(*mut ActorValueOwner, ActorAttribute) -> f32
> = GameRef::new();
#[no_mangle]
static player_avo_get_current_entry: GameRef<
    unsafe extern "system" fn(*mut ActorValueOwner, c_int) -> f32
> = GameRef::new();
#[no_mangle]
static player_avo_mod_base_entry: GameRef<
    unsafe extern "system" fn(*mut ActorValueOwner, ActorAttribute, f32)
> = GameRef::new();
#[no_mangle]
static player_avo_mod_current_entry: GameRef<
    unsafe extern "system" fn(*mut ActorValueOwner, u32, ActorAttribute, f32)
> = GameRef::new();

core_util::disarray! {
    ///
    /// Holds the relocatable locations of each object/function exposed by this file.
    ///
    /// Used by the patcher to locate our objects/functions.
    ///
    pub static GAME_SIGNATURES: [Descriptor; NUM_GAME_SIGNATURES] = [
        Descriptor::Object {
            name: "fEnchantingSkillCostBase",
            loc: GameLocation::Base { se: 506021, ae: 375771 },
            result: ENCHANTING_SKILL_COST_BASE.inner()
        },

        Descriptor::Object {
            name: "fEnchantingSkillCostMult",
            loc: GameLocation::Base { se: 506023, ae: 375774 },
            result: ENCHANTING_SKILL_COST_MULT.inner()
        },

        Descriptor::Object {
            name: "fEnchantingSkillCostScale",
            loc: GameLocation::Base { se: 506025, ae: 375777 },
            result: ENCHANTING_SKILL_COST_SCALE.inner()
        },

        Descriptor::Object {
            name: "fEnchantingCostExponent",
            loc: GameLocation::Base { se: 506027, ae: 375780 },
            result: ENCHANTING_COST_EXPONENT.inner()
        },

        Descriptor::Object {
            name: "fXPPerSkillRank",
            loc: GameLocation::Base { se: 505484, ae: 374914 },
            result: XP_PER_SKILL_RANK.inner()
        },

        Descriptor::Object {
            name: "fLegendarySkillResetValue",
            loc: GameLocation::Base { se: 507065, ae: 377771 },
            result: LEGENDARY_SKILL_RESET_VALUE.inner()
        },

        Descriptor::Object {
            name: "g_thePlayer",
            loc: GameLocation::Base { se: 517014, ae: 403521 },
            result: PLAYER_OBJECT.inner()
        },

        Descriptor::Function {
            name: "GetLevel",
            loc: GameLocation::Base { se: 36344, ae: 37334 },
            result: get_level_entry.inner()
        },

        Descriptor::Function {
            name: "PlayerAVOGetBase",
            loc: GameLocation::Base { se: 37519, ae: 38464 },
            result: player_avo_get_base_entry.inner()
        },

        Descriptor::Function {
            name: "PlayerAVOGetCurrent",
            loc: GameLocation::Base { se: 37517, ae: 38462 },
            result: player_avo_get_current_entry.inner()
        },

        Descriptor::Function {
            name: "PlayerAVOModBase",
            loc: GameLocation::Base { se: 37521, ae: 38466 },
            result: player_avo_mod_base_entry.inner()
        },

        Descriptor::Function {
            name: "PlayerAVOModCurrent",
            loc: GameLocation::Base { se: 37522, ae: 38467 },
            result: player_avo_mod_current_entry.inner()
        }
    ];
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Unwind nets for native game functions
////////////////////////////////////////////////////////////////////////////////////////////////////
//
// It is undefined behavior for a C++ function to unwind the stack into Rust code. In the current
// implementation of Rust, this just so happens to be fine, as Rust uses the same unwinding
// convention as C++. This may not always be the case, however, and so we define these wrappers to
// prevent undefined behavior from occuring.
//
// Eventually, the "system-unwind" ABI will be stablized and these wrappers will become
// unnecessary, as the game functions can simply be declared as "system-unwind" and the compiler
// will automatically abort when an unwind occurs within Rust code (since we use aborting panics).

// C++ wrappers, which catch any exceptions and redirect to us in a defined way.
extern "system" {
    fn get_level_net(player: *mut PlayerCharacter) -> u16;
    fn player_avo_get_base_net(av: *mut ActorValueOwner, attr: c_int) -> f32;
    fn player_avo_get_current_net(
        av: *mut ActorValueOwner,
        attr: c_int,
        is_se: bool,
        patch_en: bool
    ) -> f32;
    fn player_avo_mod_base_net(av: *mut ActorValueOwner, attr: c_int, delta: f32);
    fn player_avo_mod_current_net(
        av: *mut ActorValueOwner,
        unk1: u32,
        attr: c_int,
        delta: f32
    );
    pub (in crate) fn update_skill_list_unchecked(unk: *mut ());
}

/// Gets the current actor value by passing through to the original function.
pub unsafe fn avo_get_current_unchecked(
    av: *mut ActorValueOwner,
    attr: c_int
) -> f32 {
    player_avo_get_current_net(
        av,
        attr,
        skse64::version::current_runtime() <= skse64::version::RUNTIME_VERSION_1_5_97,
        SETTINGS.general.skill_formula_caps_en.get()
    )
}

/// Handles a C++ exception by just panicking.
#[no_mangle]
unsafe extern "system" fn handle_ffi_exception(
    s: *const u8,
    len: usize
) -> ! {
    // Note that this function is only given ASCII text, so we can do an unchecked conversion.
    panic!(
        "An exception occured while executing a native game function: {}",
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(s, len))
    );
}
