//!
//! @file reloc.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Exposes game objects and functions which must be located at runtime.
//! @bug No known bugs.
//!
//! FIXME: Every game function call needs a try-catch wrapper
//!

use std::ffi::{c_char, c_int};

use skyrim_patcher::{GameRef, Descriptor, GameLocation};

use super::{PlayerCharacter, ActorValueOwner, SettingCollectionMap, ActorAttribute, Setting};
use crate::hook_wrappers::player_avo_get_current_original_wrapper;
use crate::settings;

/// Gets a game setting, given a string literal.
macro_rules! game_setting {
    ( $str:literal ) => {{
        let s = ::std::concat!($str, "\0").as_bytes();
        let s = unsafe {
            ::std::slice::from_raw_parts::<'static, ::std::ffi::c_char>(
                s.as_ptr() as *const ::std::ffi::c_char,
                s.len()
            )
        };
        $crate::skyrim::get_game_setting(s)
    }}
}
pub (in crate) use game_setting;

// Game objects.
static PLAYER_OBJECT: GameRef<*mut *mut PlayerCharacter> = GameRef::new();
static GAME_SETTINGS_OBJECT: GameRef<*mut *mut SettingCollectionMap> = GameRef::new();

// Game functions. These are wrapped by safe implementations later.
static GET_LEVEL_ENTRY: GameRef<fn(*mut PlayerCharacter) -> u16> = GameRef::new();
static GET_GAME_SETTING_ENTRY: GameRef<
    fn(*mut SettingCollectionMap, *const c_char) -> *mut Setting
> = GameRef::new();
static PLAYER_AVO_GET_BASE_ENTRY: GameRef<
    fn(*mut ActorValueOwner, ActorAttribute) -> f32
> = GameRef::new();
static PLAYER_AVO_GET_CURRENT_ENTRY: GameRef<
    fn(*mut ActorValueOwner, c_int) -> f32
> = GameRef::new();
static PLAYER_AVO_MOD_BASE_ENTRY: GameRef<
    fn(*mut ActorValueOwner, ActorAttribute, f32)
> = GameRef::new();
static PLAYER_AVO_MOD_CURRENT_ENTRY: GameRef<
    fn(*mut ActorValueOwner, u32, ActorAttribute, f32)
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

/// Gets the player actor value owner structure.
pub fn get_player_avo() -> *mut ActorValueOwner {
    unsafe {
        // SAFETY: The GameRef struct ensures our player pointer is valid.
        (*(PLAYER_OBJECT.get())).as_ref().unwrap().get_avo()
    }
}

/// Gets the current level of the player.
pub fn get_player_level() -> u32 {
    let player = unsafe { *(PLAYER_OBJECT.get()) };
    (GET_LEVEL_ENTRY.get())(player) as u32
}

/// Gets the game setting associated with the null-terminated c-string.
pub fn get_game_setting(
    var: &[c_char]
) -> &'static Setting {
    assert!(var[var.len() - 1] == b'\0' as c_char);

    let settings = unsafe { *(GAME_SETTINGS_OBJECT.get()) };
    unsafe {
        // SAFETY: We have ensured our var string and settings map are valid.
        (GET_GAME_SETTING_ENTRY.get())(settings, var.as_ptr()).as_ref().unwrap()
    }
}

/// Gets the base value of a player attribute.
#[no_mangle]
pub extern "system" fn player_avo_get_base(
    attr: ActorAttribute
) -> f32 {
    (PLAYER_AVO_GET_BASE_ENTRY.get())(get_player_avo(), attr)
}

///
/// Gets the current value of a player attribute, ignoring any skill formula caps.
///
/// In order to use this function safely, the given AVO and attr must be valid.
///
pub unsafe fn player_avo_get_current_original(
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

/// Modifies the base value of a player attribute.
pub fn player_avo_mod_base(
    attr: ActorAttribute,
    val: f32
) {
    (PLAYER_AVO_MOD_BASE_ENTRY.get())(get_player_avo(), attr, val)
}

/// Modifies the current value of a player attribute.
pub fn player_avo_mod_current(
    attr: ActorAttribute,
    val: f32
) {
    // No idea what second arg does; just match game calls.
    (PLAYER_AVO_MOD_CURRENT_ENTRY.get())(get_player_avo(), 0, attr, val)
}
