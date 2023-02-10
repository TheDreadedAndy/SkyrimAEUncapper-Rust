//!
//! @file game_objects.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Exposes game objects and functions which must be located at runtime.
//! @bug No known bugs.
//!

use std::ffi::c_int;

use skyrim_patcher::{GameRef, Descriptor, GameLocation, IdLocation};

use super::{PlayerCharacter, PlayerSkills};
use super::{ActorValueOwner, ActorAttribute};
use crate::settings;

// Game objects.
static PLAYER_OBJECT: GameRef<*mut *mut PlayerCharacter> = GameRef::new();
pub static ENCHANTING_SKILL_COST_BASE: GameRef<&'static f32> = GameRef::new();
pub static ENCHANTING_SKILL_COST_SCALE: GameRef<&'static f32> = GameRef::new();
pub static ENCHANTING_COST_EXPONENT: GameRef<&'static f32> = GameRef::new();
pub static ENCHANTING_SKILL_COST_MULT: GameRef<&'static f32> = GameRef::new();
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

disarray::disarray! {
    ///
    /// Holds the relocatable locations of each object/function exposed by this file.
    ///
    /// Used by the patcher to locate our objects/functions.
    ///
    pub static GAME_SIGNATURES: [Descriptor; NUM_GAME_SIGNATURES] = [
        Descriptor::Object {
            name: "fEnchantingSkillCostBase",
            loc: GameLocation::Id(IdLocation::Base { se: 506021, ae: 375771 }),
            result: ENCHANTING_SKILL_COST_BASE.inner()
        },

        Descriptor::Object {
            name: "fEnchantingSkillCostMult",
            loc: GameLocation::Id(IdLocation::Base { se: 506023, ae: 375774 }),
            result: ENCHANTING_SKILL_COST_MULT.inner()
        },

        Descriptor::Object {
            name: "fEnchantingSkillCostScale",
            loc: GameLocation::Id(IdLocation::Base { se: 506025, ae: 375777 }),
            result: ENCHANTING_SKILL_COST_SCALE.inner()
        },

        Descriptor::Object {
            name: "fEnchantingCostExponent",
            loc: GameLocation::Id(IdLocation::Base { se: 506027, ae: 375780 }),
            result: ENCHANTING_COST_EXPONENT.inner()
        },

        Descriptor::Object {
            name: "fLegendarySkillResetValue",
            loc: GameLocation::Id(IdLocation::Base { se: 507065, ae: 377771 }),
            result: LEGENDARY_SKILL_RESET_VALUE.inner()
        },

        Descriptor::Object {
            name: "g_thePlayer",
            loc: GameLocation::Id(IdLocation::Base { se: 517014, ae: 403521 }),
            result: PLAYER_OBJECT.inner()
        },

        Descriptor::Function {
            name: "GetLevel",
            loc: GameLocation::Id(IdLocation::Base { se: 36344, ae: 37334 }),
            result: get_level_entry.inner()
        },

        Descriptor::Function {
            name: "PlayerAVOGetBase",
            loc: GameLocation::Id(IdLocation::Base { se: 37519, ae: 38464 }),
            result: player_avo_get_base_entry.inner()
        },

        Descriptor::Function {
            name: "PlayerAVOGetCurrent",
            loc: GameLocation::Id(IdLocation::Base { se: 37517, ae: 38462 }),
            result: player_avo_get_current_entry.inner()
        },

        Descriptor::Function {
            name: "PlayerAVOModBase",
            loc: GameLocation::Id(IdLocation::Base { se: 37521, ae: 38466 }),
            result: player_avo_mod_base_entry.inner()
        },

        Descriptor::Function {
            name: "PlayerAVOModCurrent",
            loc: GameLocation::Id(IdLocation::Base { se: 37522, ae: 38467 }),
            result: player_avo_mod_current_entry.inner()
        }
    ];
}

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
    fn improve_player_skill_points_net(
        data: *mut PlayerSkills,
        attr: c_int,
        exp: f32,
        unk1: u64,
        unk2: u32,
        natural_exp: bool,
        unk4: bool
    );
}

/// Handles a C++ exception by just panicking.
#[no_mangle]
unsafe extern "system" fn handle_ffi_exception(
    s: *const u8,
    len: usize
) -> ! {
    panic!(
        "An exception occured while executing a native game function: {}",
        std::str::from_utf8(std::slice::from_raw_parts(s, len)).unwrap()
    );
}

/// Helper for assembly code to get the player pointer.
#[no_mangle]
extern "system" fn get_player() -> *mut PlayerCharacter {
    unsafe {
        // SAFETY: The GameRef struct ensures our player pointer is valid.
        *(PLAYER_OBJECT.get())
    }
}

/// Gets the player actor value owner structure.
#[no_mangle]
pub extern "system" fn get_player_avo() -> *mut ActorValueOwner {
    unsafe {
        // SAFETY: The GameRef struct ensures our player pointer is valid.
        (*get_player()).get_avo()
    }
}

/// Gets a reference to the players perk pool.
pub fn get_player_perk_pool() -> &'static core::cell::Cell<u8> {
    unsafe {
        // SAFETY: The GameRef struct ensures our player pointer is valid.
        (*get_player()).get_perk_pool()
    }
}

/// Gets the current level of the player.
pub fn get_player_level() -> u32 {
    unsafe { get_level_net(*(PLAYER_OBJECT.get())) as u32 }
}

///
/// Gets the base value of a player attribute.
///
/// In order to use this function safely, the given attribute and avo must be valid.
///
#[no_mangle]
pub unsafe extern "system" fn player_avo_get_base_unchecked(
    av: *mut ActorValueOwner,
    attr: c_int
) -> f32 {
    player_avo_get_base_net(av, attr)
}

///
/// Gets the current value of a player attribute, ignoring any skill formula caps.
///
/// In order to use this function safely, the given AVO and attr must be valid.
///
#[no_mangle]
pub unsafe extern "system" fn player_avo_get_current_unchecked(
    av: *mut ActorValueOwner,
    attr: c_int
) -> f32 {
    let is_se = skse64::version::current_runtime() <= skse64::version::RUNTIME_VERSION_1_5_97;
    player_avo_get_current_net(av, attr, is_se, settings::is_skill_formula_cap_enabled())
}

/// Gets the base value of the given attribute.
pub fn player_avo_get_base(
    attr: ActorAttribute
) -> f32 {
    unsafe { player_avo_get_base_unchecked(get_player_avo(), attr as c_int) }
}

/// Gets the current value of the given attribute.
pub fn player_avo_get_current(
    attr: ActorAttribute
) -> f32 {
    unsafe { player_avo_get_current_unchecked(get_player_avo(), attr as c_int) }
}

/// Modifies the base value of a player attribute.
pub fn player_avo_mod_base(
    attr: ActorAttribute,
    val: f32
) {
    unsafe { player_avo_mod_base_net(get_player_avo(), attr as c_int, val) }
}

/// Modifies the current value of a player attribute.
pub fn player_avo_mod_current(
    attr: ActorAttribute,
    val: f32
) {
    // No idea what second arg does; just match game calls.
    unsafe { player_avo_mod_current_net(get_player_avo(), 0, attr as c_int, val) }
}

/// Improves the skill experience of a player skill.
#[no_mangle]
pub unsafe extern "system" fn improve_player_skill_points(
    data: *mut PlayerSkills,
    attr: c_int,
    exp: f32,
    unk1: u64, // r9
    unk2: u32, // 0x70(%rsp)
    natural_exp: bool, // true if natural, false if by training.
    unk4: bool // true if by training and not the end of the count. Controls advancement display?
) {
    unsafe { improve_player_skill_points_net(data, attr, exp, unk1, unk2, natural_exp, unk4); }
}
