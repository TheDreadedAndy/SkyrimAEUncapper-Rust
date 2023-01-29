//!
//! @file hook_wrappers.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Exposes assembly wrappers used by patches.
//! @bug No known bugs.
//!

use std::ffi::c_int;

use crate::skyrim::{ActorAttribute, ActorValueOwner, PlayerSkills};

extern "system" {
    pub fn skill_cap_patch_wrapper();
    pub fn calculate_charge_points_per_use_wrapper();
    pub fn player_avo_get_current_original_wrapper(
        av: *mut ActorValueOwner,
        attr: c_int
    ) -> f32;
    pub fn display_true_skill_level_hook();
    pub fn display_true_skill_color_hook();
    pub fn improve_level_exp_by_skill_level_wrapper();
    pub fn improve_player_skill_points_original(
        skill_data: *mut PlayerSkills,
        skill: ActorAttribute,
        exp: f32,
        unk1: u64,
        unk2: u32,
        unk3: u8,
        unk4: bool
    );
    pub fn modify_perk_pool_wrapper();
    pub fn legendary_reset_skill_level_wrapper();
    pub fn check_condition_for_legendary_skill_wrapper();
    pub fn check_condition_for_legendary_skill_alt_wrapper();
    pub fn hide_legendary_button_wrapper();
}
