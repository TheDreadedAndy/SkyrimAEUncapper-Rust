//!
//! @file hook_wrappers.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Exposes assembly wrappers used by patches.
//! @bug No known bugs.
//!

extern "system" {
    pub fn skill_cap_patch_wrapper_ae();
    pub fn skill_cap_patch_wrapper_se();
    pub fn max_charge_begin_wrapper_ae();
    pub fn max_charge_begin_wrapper_se();
    pub fn max_charge_end_wrapper_ae();
    pub fn max_charge_end_wrapper_se();
    pub fn calculate_charge_points_per_use_wrapper_ae();
    pub fn calculate_charge_points_per_use_wrapper_se();
    pub fn display_true_skill_level_hook_ae();
    pub fn display_true_skill_level_hook_se();
    pub fn display_true_skill_color_hook();
    pub fn improve_player_skill_points_wrapper_ae();
    pub fn improve_player_skill_points_wrapper_se();
    pub fn improve_level_exp_by_skill_level_wrapper_ae();
    pub fn improve_level_exp_by_skill_level_wrapper_se();
    pub fn improve_attribute_when_level_up_wrapper();
    pub fn modify_perk_pool_wrapper_ae();
    pub fn modify_perk_pool_wrapper_se();
    pub fn legendary_reset_skill_level_wrapper();
    pub fn check_condition_for_legendary_skill_wrapper();
    pub fn hide_legendary_button_wrapper_ae();
    pub fn hide_legendary_button_wrapper_se();
    pub fn clear_legendary_button_wrapper_ae();
    pub fn clear_legendary_button_wrapper_se();
}

core::arch::global_asm! {
    include_str!("hook_wrappers.S"),
    options(att_syntax)
}
