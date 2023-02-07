//!
//! @file hook_wrappers.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Exposes assembly wrappers used by patches.
//! @bug No known bugs.
//!

extern "system" {
    pub fn skill_cap_patch_wrapper();
    pub fn max_charge_begin_wrapper();
    pub fn max_charge_end_wrapper();
    pub fn calculate_charge_points_per_use_wrapper();
    pub fn display_true_skill_level_hook();
    pub fn display_true_skill_color_hook();
    pub fn improve_level_exp_by_skill_level_wrapper();
    pub fn improve_attribute_when_level_up_wrapper();
    pub fn improve_skill_by_training_hook();
    pub fn modify_perk_pool_wrapper();
    pub fn legendary_reset_skill_level_wrapper();
    pub fn check_condition_for_legendary_skill_wrapper();
    pub fn check_condition_for_legendary_skill_alt_wrapper();
    pub fn hide_legendary_button_wrapper();
    pub fn clear_legendary_button_wrapper();
}

core::arch::global_asm! {
    include_str!("hook_wrappers.S"),
    options(att_syntax)
}
