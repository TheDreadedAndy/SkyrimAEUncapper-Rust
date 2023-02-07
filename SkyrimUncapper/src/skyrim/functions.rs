//!
//! @file reloc.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Exposes game objects and functions which must be located at runtime.
//! @bug No known bugs.
//!

use std::ffi::{c_char, c_int};

use skyrim_patcher::{GameRef, Descriptor, GameLocation};

use super::{PlayerCharacter, ActorValueOwner, ActorAttribute, SettingCollectionMap, Setting};
use crate::hook_wrappers::player_avo_get_current_original_wrapper;
use crate::settings;

/// Gets a game setting, given a string literal.
macro_rules! game_setting {
    ( $str:literal ) => {{
        let s = ::std::concat!($str, "\0").as_bytes().as_ptr() as *const ::std::ffi::c_char;
        unsafe { $crate::skyrim::get_game_setting(s) }
    }}
}
pub (in crate) use game_setting;

// Game objects.
static PLAYER_OBJECT: GameRef<*mut *mut PlayerCharacter> = GameRef::new();
static GAME_SETTINGS_OBJECT: GameRef<*mut *mut SettingCollectionMap> = GameRef::new();

// Game functions. These are wrapped by C++ catchers and then safe implementations later.
#[no_mangle]
static GET_LEVEL_ENTRY: GameRef<
    unsafe extern "system-unwind" fn(*mut PlayerCharacter) -> u16
> = GameRef::new();
#[no_mangle]
static GET_GAME_SETTING_ENTRY: GameRef<
    unsafe extern "system-unwind" fn(*mut SettingCollectionMap, *const c_char) -> *mut Setting
> = GameRef::new();
#[no_mangle]
static PLAYER_AVO_GET_BASE_ENTRY: GameRef<
    unsafe extern "system-unwind" fn(*mut ActorValueOwner, c_int) -> f32
> = GameRef::new();
#[no_mangle]
static PLAYER_AVO_GET_CURRENT_ENTRY: GameRef<
    unsafe extern "system-unwind" fn(*mut ActorValueOwner, c_int) -> f32
> = GameRef::new();
#[no_mangle]
static PLAYER_AVO_MOD_BASE_ENTRY: GameRef<
    unsafe extern "system-unwind" fn(*mut ActorValueOwner, c_int, f32)
> = GameRef::new();
#[no_mangle]
static PLAYER_AVO_MOD_CURRENT_ENTRY: GameRef<
    unsafe extern "system-unwind" fn(*mut ActorValueOwner, u32, c_int, f32)
> = GameRef::new();

disarray::disarray! {
    ///
    /// Holds the relocatable locations of each object/function exposed by this file.
    ///
    /// Used by the patcher to locate our objects/functions.
    ///
    pub static GAME_SIGNATURES: [Descriptor; NUM_GAME_SIGNATURES] = [
        Descriptor::Object {
            name: "g_thePlayer",
            loc: GameLocation::Id { id: 403521, offset: 0 },
            result: PLAYER_OBJECT.inner()
        },

        Descriptor::Object {
            name: "g_gameSettingCollection",
            loc: GameLocation::Id { id: 400782, offset: 0 },
            result: GAME_SETTINGS_OBJECT.inner()
        },

        Descriptor::Function {
            name: "GetLevel",
            loc: GameLocation::Id { id: 37334, offset: 0 },
            result: GET_LEVEL_ENTRY.inner()
        },

        Descriptor::Function {
            name: "GetGameSetting",
            loc: GameLocation::Id { id: 22788, offset: 0 },
            result: GET_GAME_SETTING_ENTRY.inner()
        },

        Descriptor::Function {
            name: "PlayerAVOGetBase",
            loc: GameLocation::Id { id: 38464, offset: 0 },
            result: PLAYER_AVO_GET_BASE_ENTRY.inner()
        },

        Descriptor::Function {
            name: "PlayerAVOGetCurrent",
            loc: GameLocation::Id { id: 38462, offset: 0 },
            result: PLAYER_AVO_GET_CURRENT_ENTRY.inner()
        },

        Descriptor::Function {
            name: "PlayerAVOModBase",
            loc: GameLocation::Id { id: 38466, offset: 0 },
            result: PLAYER_AVO_MOD_BASE_ENTRY.inner()
        },

        Descriptor::Function {
            name: "PlayerAVOModCurrent",
            loc: GameLocation::Id { id: 38467, offset: 0 },
            result: PLAYER_AVO_MOD_CURRENT_ENTRY.inner()
        }
    ];
}

/// Helper for assembly code to get the player pointer.
#[no_mangle]
extern "system-unwind" fn get_player() -> *mut PlayerCharacter {
    unsafe {
        // SAFETY: The GameRef struct ensures our player pointer is valid.
        *(PLAYER_OBJECT.get())
    }
}

/// Gets the player actor value owner structure.
#[no_mangle]
pub extern "system-unwind" fn get_player_avo() -> *mut ActorValueOwner {
    unsafe {
        // SAFETY: The GameRef struct ensures our player pointer is valid.
        (*get_player()).get_avo()
    }
}

/// Gets a reference to the players perk pool.
pub extern "system-unwind" fn get_player_perk_pool() -> &'static core::cell::Cell<u8> {
    unsafe {
        // SAFETY: The GameRef struct ensures our player pointer is valid.
        (*get_player()).get_perk_pool()
    }
}

/// Gets the current level of the player.
pub extern "system-unwind" fn get_player_level() -> u32 {
    unsafe { (GET_LEVEL_ENTRY.get())(*(PLAYER_OBJECT.get())) as u32 }
}

/// Gets the game setting associated with the null-terminated c-string.
pub unsafe extern "system-unwind" fn get_game_setting(
    var: *const c_char
) -> &'static Setting {
    let settings = unsafe { *(GAME_SETTINGS_OBJECT.get()) };
    unsafe {
        // SAFETY: We have ensured our var string and settings map are valid.
        (GET_GAME_SETTING_ENTRY.get())(settings, var).as_ref().unwrap()
    }
}

///
/// Gets the base value of a player attribute.
///
/// In order to use this function safely, the given attribute and avo must be valid.
///
#[no_mangle]
pub unsafe extern "system-unwind" fn player_avo_get_base_unchecked(
    av: *mut ActorValueOwner,
    attr: c_int
) -> f32 {
    (PLAYER_AVO_GET_BASE_ENTRY.get())(av, attr)
}

///
/// Gets the current value of a player attribute, ignoring any skill formula caps.
///
/// In order to use this function safely, the given AVO and attr must be valid.
///
#[no_mangle]
pub unsafe extern "system-unwind" fn player_avo_get_current_unchecked(
    av: *mut ActorValueOwner,
    attr: c_int
) -> f32 {
    if settings::is_skill_formula_cap_enabled() {
        // Patch installed, so we need to use the wrapper.
        player_avo_get_current_original_wrapper(av, attr)
    } else {
        // No patch, so we can just call the og function
        // (and must, since we don't have a trampoline).
        (PLAYER_AVO_GET_CURRENT_ENTRY.get())(av, attr)
    }
}

/// Gets the base value of the given attribute.
pub extern "system-unwind" fn player_avo_get_base(
    attr: ActorAttribute
) -> f32 {
    unsafe { player_avo_get_base_unchecked(get_player_avo(), attr as c_int) }
}

/// Gets the current value of the given attribute.
pub extern "system-unwind" fn player_avo_get_current(
    attr: ActorAttribute
) -> f32 {
    unsafe { player_avo_get_current_unchecked(get_player_avo(), attr as c_int) }
}

/// Modifies the base value of a player attribute.
pub extern "system-unwind" fn player_avo_mod_base(
    attr: ActorAttribute,
    val: f32
) {
    unsafe { (PLAYER_AVO_MOD_BASE_ENTRY.get())(get_player_avo(), attr as c_int, val) }
}

/// Modifies the current value of a player attribute.
pub extern "system-unwind" fn player_avo_mod_current(
    attr: ActorAttribute,
    val: f32
) {
    // No idea what second arg does; just match game calls.
    unsafe { (PLAYER_AVO_MOD_CURRENT_ENTRY.get())(get_player_avo(), 0, attr as c_int, val) }
}

pub use crate::hook_wrappers::improve_player_skill_points;
