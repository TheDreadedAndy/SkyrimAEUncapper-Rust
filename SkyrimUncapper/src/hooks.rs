//!
//! @file hooks.rs
//! @author Andrew Spaulding (Kasplat)
//! @author Vadfromnu
//! @author Kassent
//! @brief Rust implementation of game patch hooks.
//! @bug No known bugs
//!
//! Note that each function in this file is either called by the game or by
//! an assembly wrapper, so they must be declared extern system.
//!

use std::ffi::c_int;

use skse64::errors::skse_assert;

use crate::settings;
use crate::skyrim::{ActorAttribute, ActorValueOwner, PlayerSkills};
use crate::skyrim::{player_avo_get_current_original, get_game_setting};

/// Formats a string as a game variable string.
macro_rules! game_var {
    ( $str:literal ) => {
        ::std::concat!($str, "\0").as_bytes()
    }
}

/// Determines the real skill cap of the given skill.
#[no_mangle]
extern "system" fn get_skill_cap_hook(
    skill: c_int
) -> f32 {
    skse_assert!(settings::is_skill_cap_enabled());
    settings::get_skill_cap(ActorAttribute::from_raw(skill).unwrap())
}

///
/// Reimplements the enchantment charge point equation.
///
/// The original equation would fall apart for levels above 199, so this
/// implementation caps the level in the calculation to 199.
///
#[no_mangle]
extern "system" fn calculate_charge_points_per_use_hook(
    av: *mut ActorValueOwner,
    base_points: f32,
    max_charge: f32
) -> f32 {
    0.0
}

/// Caps the formula results for each skill.
extern "system" fn player_avo_get_current_hook(
    av: *mut ActorValueOwner,
    attr: c_int
) -> f32 {
    0.0
}

/// Applies a multiplier to the exp gain for the given skill.
extern "system" fn improve_player_skill_points_hook(
    skill_data: *mut PlayerSkills,
    attr: c_int,
    exp: f32,
    unk1: u64,
    unk2: u32,
    unk3: u8,
    unk4: bool
) {
}

/// Adjusts the number of perks the player recieves at level-up.
#[no_mangle]
extern "system" fn modify_perk_pool_hook(
    points: u8,
    delta: i8
) -> u8 {
    0
}

/// Multiplies the exp gain of a level-up by the configured multiplier.
#[no_mangle]
extern "system" fn improve_level_exp_by_skill_level_hook(
    exp: f32,
    attr: c_int
) {
}

///
/// Adjusts the attribute gain at each level-up based on the configured settings.
///
/// This function overwrites a call to player_avo->mod_base(). Since we're overwriting
/// a call, we don't need to reg save and, thus, don't need a wrapper. We also overwrite
/// the carry weight level-up code.
///
extern "system" fn improve_attribute_when_level_up_hook(
    av: *mut ActorValueOwner,
    choice: c_int
) {
}

/// Determines what level a skill should take on after being legendary'd.
///
/// FIXME: Breaks legendarying with the space bar? Or is that the next one?
#[no_mangle]
extern "system" fn legendary_reset_skill_level_hook(
    base_level: f32
) {
}

/// Overwrites the check which determines when a skill can be legendary'd.
#[no_mangle]
extern "system" fn check_condition_for_legendary_skill_hook(
    _av: *mut ActorValueOwner,
    skill: c_int
) -> bool {
    false
}

/// Determines if the legendary button should be displayed for the given skill.
#[no_mangle]
extern "system" fn hide_legendary_button_hook(
    _av: *mut ActorValueOwner,
    skill: c_int
) -> bool {
    false
}
