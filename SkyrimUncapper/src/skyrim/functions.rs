//!
//! @file reloc.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Exposes game objects and functions which must be located at runtime.
//! @bug No known bugs.
//!

use std::ffi::{c_char, c_int};

use skyrim_patcher::{GameRef, Descriptor, GameLocation};

use super::{PlayerCharacter, ActorValueOwner, SettingCollectionMap, ActorAttribute, Setting};
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

// Game functions. These are wrapped by C++ catchers and then safe implementations later.
#[no_mangle]
static get_level_entry: GameRef<fn(*mut PlayerCharacter) -> u16> = GameRef::new();
#[no_mangle]
static get_game_setting_entry: GameRef<
    fn(*mut SettingCollectionMap, *const c_char) -> *mut Setting
> = GameRef::new();
#[no_mangle]
static player_avo_get_base_entry: GameRef<
    fn(*mut ActorValueOwner, ActorAttribute) -> f32
> = GameRef::new();
#[no_mangle]
static player_avo_get_current_entry: GameRef<
    fn(*mut ActorValueOwner, c_int) -> f32
> = GameRef::new();
#[no_mangle]
static player_avo_mod_base_entry: GameRef<
    fn(*mut ActorValueOwner, ActorAttribute, f32)
> = GameRef::new();
#[no_mangle]
static player_avo_mod_current_entry: GameRef<
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
            result: get_level_entry.inner()
        },

        Descriptor::Function {
            name: "GetGameSetting",
            loc: GameLocation::Id { id: 22788, offset: 0 },
            result: get_game_setting_entry.inner()
        },

        Descriptor::Function {
            name: "PlayerAVOGetBase",
            loc: GameLocation::Id { id: 38464, offset: 0 },
            result: player_avo_get_base_entry.inner()
        },

        Descriptor::Function {
            name: "PlayerAVOGetCurrent",
            loc: GameLocation::Id { id: 38462, offset: 0 },
            result: player_avo_get_current_entry.inner()
        },

        Descriptor::Function {
            name: "PlayerAVOModBase",
            loc: GameLocation::Id { id: 38466, offset: 0 },
            result: player_avo_mod_base_entry.inner()
        },

        Descriptor::Function {
            name: "PlayerAVOModCurrent",
            loc: GameLocation::Id { id: 38467, offset: 0 },
            result: player_avo_mod_current_entry.inner()
        }
    ];
}

// C++ wrappers, which catch any exceptions and redirect to us in a defined way.
extern "system" {
    fn get_level_net(player: *mut PlayerCharacter) -> u16;
    fn get_game_setting_net(map: *mut SettingCollectionMap, setting: *const c_char) -> *mut Setting;
    fn player_avo_get_base_net(av: *mut ActorValueOwner, attr: ActorAttribute) -> f32;
    fn player_avo_get_current_net(av: *mut ActorValueOwner, attr: c_int, patch_en: bool) -> f32;
    fn player_avo_mod_base_net(av: *mut ActorValueOwner, attr: ActorAttribute, delta: f32);
    fn player_avo_mod_current_net(
        av: *mut ActorValueOwner,
        unk1: u32,
        attr: ActorAttribute,
        delta: f32
    );
}

/// Handles a C++ exception by just panicking.
#[no_mangle]
extern "system" fn handle_ffi_exception() -> ! {
    panic!("An exception occured while executing a native game function");
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

/// Gets the game setting associated with the null-terminated c-string.
pub fn get_game_setting(
    var: &[c_char]
) -> &'static Setting {
    assert!(var[var.len() - 1] == b'\0' as c_char);

    let settings = unsafe { *(GAME_SETTINGS_OBJECT.get()) };
    unsafe {
        // SAFETY: We have ensured our var string and settings map are valid.
        get_game_setting_net(settings, var.as_ptr()).as_ref().unwrap()
    }
}

/// Gets the base value of a player attribute.
#[no_mangle]
pub extern "system" fn player_avo_get_base(
    attr: ActorAttribute
) -> f32 {
    unsafe { player_avo_get_base_net(get_player_avo(), attr) }
}

///
/// Gets the current value of a player attribute, ignoring any skill formula caps.
///
/// In order to use this function safely, the given AVO and attr must be valid.
///
#[no_mangle]
pub unsafe extern "system" fn player_avo_get_current(
    av: *mut ActorValueOwner,
    attr: c_int
) -> f32 {
    unsafe { player_avo_get_current_net(av, attr, settings::is_skill_formula_cap_enabled()) }
}

/// Modifies the base value of a player attribute.
pub fn player_avo_mod_base(
    attr: ActorAttribute,
    val: f32
) {
    unsafe { player_avo_mod_base_net(get_player_avo(), attr, val) }
}

/// Modifies the current value of a player attribute.
pub fn player_avo_mod_current(
    attr: ActorAttribute,
    val: f32
) {
    // No idea what second arg does; just match game calls.
    unsafe { player_avo_mod_current_net(get_player_avo(), 0, attr, val) }
}
