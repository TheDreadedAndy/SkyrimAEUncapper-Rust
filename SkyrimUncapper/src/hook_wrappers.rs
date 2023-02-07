//!
//! @file hook_wrappers.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Exposes assembly wrappers used by patches.
//! @bug No known bugs.
//!

use core::ffi::c_int;
use crate::skyrim::{ActorValueOwner, PlayerSkills};

// Not unwind safe (violates unwind convention).
extern "system" {
    pub fn max_charge_begin_wrapper();
}

extern "system-unwind" {
    pub fn display_true_skill_level_hook();
    pub fn display_true_skill_color_hook();
    pub fn improve_skill_by_training_hook();

    pub fn skill_cap_patch_wrapper();
    pub fn max_charge_end_wrapper();
    pub fn calculate_charge_points_per_use_wrapper();
    pub fn improve_level_exp_by_skill_level_wrapper();
    pub fn improve_attribute_when_level_up_wrapper();
    pub fn modify_perk_pool_wrapper();
    pub fn legendary_reset_skill_level_wrapper();
    pub fn check_condition_for_legendary_skill_wrapper();
    pub fn check_condition_for_legendary_skill_alt_wrapper();
    pub fn hide_legendary_button_wrapper();
    pub fn clear_legendary_button_wrapper();

    pub fn player_avo_get_current_original_wrapper(av: *mut ActorValueOwner, attr: c_int) -> f32;
    pub fn improve_player_skill_points(
        data: *mut PlayerSkills,
        attr: c_int,
        exp: f32,
        unk1: u64,
        unk2: u32,
        unk3: u8,
        unk4: bool
    );
}

core::arch::global_asm! {
    include_str!("hook_wrappers.S"),
    options(att_syntax)
}
