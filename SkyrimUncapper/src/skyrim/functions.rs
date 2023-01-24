//!
//! @file reloc.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Exposes game objects and functions which must be located at runtime.
//! @bug No known bugs.
//!

use std::ffi::c_char;

use super::{PlayerCharacter, SettingsCollectionMap, ActorAttribute, Setting};
use crate::patcher::{RelocAddr, RelocPatch, GameLocation};

// Game objects.
static PLAYER_OBJECT: RelocAddr<*mut *mut PlayerCharacter> = RelocAddr::new();
static GAME_SETTINGS_OBJECT: RelocAddr<*mut *mut SettingsCollectionMap> = RelocAddr::new();

// Game functions. These are wrapped by safe implementations later.
static GET_LEVEL_ENTRY: RelocAddr<fn(*mut ()) -> u16> = RelocAddr::new();
static GET_GAME_SETTING_ENTRY: RelocAddr<
    fn(*mut SettingsCollectionMap, *const c_char) -> *mut Setting
> = RelocAddr::new();
static PLAYER_AVO_GET_BASE_ENTRY: RelocAddr<
    fn(*mut PlayerCharacter, ActorAttribute) -> f32
> = RelocAddr::new();
static PLAYER_AVO_GET_CURRENT_ENTRY: RelocAddr<
    fn(*mut PlayerCharacter, ActorAttribute) -> f32
> = RelocAddr::new();
static PLAYER_AVO_MOD_BASE_ENTRY: RelocAddr<
    fn(*mut PlayerCharacter, ActorAttribute, f32)
> = RelocAddr::new();
static PLAYER_AVO_MOD_CURRENT_ENTRY: RelocAddr<
    fn(*mut PlayerCharacter, u32, ActorAttribute, f32)
> = RelocAddr::new();

///
/// Holds the relocatable locations of each object/function exposed by this file.
///
/// Used by the patcher to locate our objects/functions.
///
pub const GAME_SIGNATURES: &'static [&'static RelocPatch] = &[
    &RelocPatch::Object {
        name: "g_thePlayer",
        loc: GameLocation::Id { id: 403521, offset: 0 },
        result: PLAYER_OBJECT.inner()
    },

    &RelocPatch::Object {
        name: "g_gameSettingCollection",
        loc: GameLocation::Id { id: 400782, offset: 0 },
        result: GAME_SETTINGS_OBJECT.inner()
    },

    &RelocPatch::Function {
        name: "GetLevel",
        loc: GameLocation::Id { id: 37334, offset: 0 },
        result: GET_LEVEL_ENTRY.inner()
    },

    &RelocPatch::Function {
        name: "GetGameSetting",
        loc: GameLocation::Id { id: 22788, offset: 0 },
        result: GET_GAME_SETTING_ENTRY.inner()
    },

    &RelocPatch::Function {
        name: "PlayerAVOGetBase",
        loc: GameLocation::Id { id: 38464, offset: 0 },
        result: PLAYER_AVO_GET_BASE_ENTRY.inner()
    },

    &RelocPatch::Function {
        name: "PlayerAVOGetCurrent",
        loc: GameLocation::Id { id: 38462, offset: 0 },
        result: PLAYER_AVO_GET_CURRENT_ENTRY.inner()
    },

    &RelocPatch::Function {
        name: "PlayerAVOModBase",
        loc: GameLocation::Id { id: 38466, offset: 0 },
        result: PLAYER_AVO_MOD_BASE_ENTRY.inner()
    },

    &RelocPatch::Function {
        name: "PlayerAVOModCurrent",
        loc: GameLocation::Id { id: 38467, offset: 0 },
        result: PLAYER_AVO_MOD_CURRENT_ENTRY.inner()
    }
];
