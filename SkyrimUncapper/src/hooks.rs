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

use core::ffi::c_int;
use core::sync::atomic::{AtomicBool, Ordering};

use skyrim_patcher::{Descriptor, Hook, Register, GameLocation, GameRef, signature};

use crate::settings::SETTINGS;
use crate::skyrim::*;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Patch wrappers
////////////////////////////////////////////////////////////////////////////////////////////////////
//
// These are the assembly entry points that ensure that the jump between skyrims game code and our
// patches is safe and well-defined. Documentation on how the individual patches accomplish this
// can be found in hook_wrappers.S.

extern "system" {
    fn skill_cap_patch_wrapper_ae();
    fn skill_cap_patch_wrapper_se();
    fn max_charge_begin_wrapper_ae();
    fn max_charge_begin_wrapper_se();
    fn max_charge_end_wrapper_ae();
    fn max_charge_end_wrapper_se();
    fn calculate_charge_points_per_use_wrapper_ae();
    fn calculate_charge_points_per_use_wrapper_se();
    fn player_avo_get_current_wrapper();
    fn update_skill_list_wrapper();
    fn improve_player_skill_points_wrapper_ae();
    fn improve_player_skill_points_wrapper_se();
    fn improve_level_exp_by_skill_level_wrapper_ae();
    fn improve_level_exp_by_skill_level_wrapper_se();
    fn improve_attribute_when_level_up_wrapper();
    fn modify_perk_pool_wrapper_ae();
    fn modify_perk_pool_wrapper_se();
    fn legendary_reset_skill_level_wrapper();
    fn check_condition_for_legendary_skill_wrapper();
    fn hide_legendary_button_wrapper_ae();
    fn hide_legendary_button_wrapper_se();
    fn clear_legendary_button_wrapper_ae();
    fn clear_legendary_button_wrapper_se();
}

core::arch::global_asm! {
    include_str!("hook_wrappers.S"),

    // These symbol declarations let us access the mangled rust functions in the asm code by simply
    // surrounding our reference to the symbol in curly braces. The benefit of this is that we
    // don't need to use the no_mangle attribute, which also globally exports the symbol.
    player_avo_get_current_return_trampoline = sym PLAYER_AVO_GET_CURRENT_RETURN_TRAMPOLINE,
    update_skill_list_return_trampoline      = sym UPDATE_SKILL_LIST_RETURN_TRAMPOLINE,

    get_skill_cap_hook                       = sym get_skill_cap_hook,
    max_charge_begin_hook                    = sym max_charge_begin_hook,
    max_charge_end_hook                      = sym max_charge_end_hook,
    calculate_charge_points_per_use_hook     = sym calculate_charge_points_per_use_hook,
    player_avo_get_current_hook              = sym player_avo_get_current_hook,
    update_skill_list_hook                   = sym update_skill_list_hook,
    improve_player_skill_points_hook         = sym improve_player_skill_points_hook,
    improve_level_exp_by_skill_level_hook    = sym improve_level_exp_by_skill_level_hook,
    modify_perk_pool_hook                    = sym modify_perk_pool_hook,
    improve_attribute_when_level_up_hook     = sym improve_attribute_when_level_up_hook,
    legendary_reset_skill_level_hook         = sym legendary_reset_skill_level_hook,
    check_condition_for_legendary_skill_hook = sym check_condition_for_legendary_skill_hook,
    hide_legendary_button_hook               = sym hide_legendary_button_hook,
    clear_legendary_button_hook              = sym clear_legendary_button_hook,

    // These are defined in the skyrim game objects file.
    player_avo_get_base_unchecked            = sym PlayerCharacter::get_base_unchecked,
    get_player_avo                           = sym PlayerCharacter::get_avo,

    options(att_syntax)
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Patch definitions
////////////////////////////////////////////////////////////////////////////////////////////////////
//
// These structures inform the skyrim patcher of where each patch is located for the various game
// versions, and how the patch should be installed.
//
// Note that, for increased compatibility, we do not use calls/jumps that would require an indirect
// or relative jump. Instead, each call/jump clobbers a register that is not currently being used.
// This means that the plugin does not contribute to the practical limit on the number of SKSE
// plugins that can be loaded at a time (due to there being a limited address space before the
// skyrim binary), but it complicates the injection of hooks as some skyrim versions are compiled
// with settings that allow the compiler to disobey calling conventions and assume that registers
// that would conventionally be considered clobbered still hold their values.
//
// We additionally define strings to inform users of known conflicts, when the patches associated
// with those conflicts are loaded by the game.

// Mods we conflict with.
const MEH_CUSTOM_SKILL : &str = "Meh321's Custom Skills Framework";
const ZAX_EXPERIENCE   : &str = "Zax's Experience";

// Conflicts for each patch group.
const LEGENDARY_CONFLICTS  : &str = MEH_CUSTOM_SKILL;
const LEVEL_MULT_CONFLICTS : &str = ZAX_EXPERIENCE;

//
// Trampolines used by hooks to return to game code.
//
// Boing!
//
static PLAYER_AVO_GET_CURRENT_RETURN_TRAMPOLINE : GameRef<usize> = GameRef::new();
static UPDATE_SKILL_LIST_RETURN_TRAMPOLINE      : GameRef<usize> = GameRef::new();

////////////////////////////////////////////////////////////////////////////////////////////////////

core_util::disarray! {
    /// The hooks which must be installed by the game patcher.
    pub static HOOK_SIGNATURES: [Descriptor; NUM_HOOK_SIGNATURES] = [
        //
        // Injects the code which alters the real skill cap of each skill.
        //
        // IMPORTANT: THIS PATCH MUST BE AT LEAST 14 BYTES LONG.
        // Note that the last two bytes of this patch must be overwritten with NOPs
        // and returned to, at the request of the author of the eXPerience mod (17751).
        // This is handled by the patcher, we need only make our signature long enough.
        //
        Descriptor::Patch {
            name: "GetSkillCap",
            enabled: || SETTINGS.general.skill_caps_en.get(),
            conflicts: None,
            hook: Hook::Call12 {
                entry: skill_cap_patch_wrapper_ae as *const u8,
                clobber: Register::Rax
            },
            loc: GameLocation::Ae { id: 41561, offset: 0x6f },
            sig: signature![
                0xff, 0x50, 0x18,
                0x44, 0x0f, 0x28, 0xc0,
                0xf3, 0x44, 0x0f, 0x10, 0x15, ?, ?, ?, ?; 16
            ]
        },
        Descriptor::Patch {
            name: "GetSkillCap",
            enabled: || SETTINGS.general.skill_caps_en.get(),
            conflicts: None,
            hook: Hook::Call12 {
                entry: skill_cap_patch_wrapper_se as *const u8,
                clobber: Register::Rax
            },
            loc: GameLocation::Se { id: 40554, offset: 0x45 },
            sig: signature![
                0xff, 0x50, 0x18,
                0xf3, 0x44, 0x0f, 0x10, 0x05, ?, ?, ?, ?,
                0x0f, 0x28, 0xf0; 15
            ]
        },

        //
        // Injects a function call to alter the behavior of player_avo_get_current()
        // for enchanting during the region of the patch and the following patch.
        // This allows us to ensure that the charge cap is used within this region,
        // thus ensuring that the charge and magnitude cap can be configured independently.
        //
        Descriptor::Patch {
            name: "BeginMaxChargeCalculation",
            enabled: || SETTINGS.general.enchanting_patch_en.get(),
            conflicts: None,
            hook: Hook::Call12 {
                entry: max_charge_begin_wrapper_ae as *const u8,
                clobber: Register::Rax // Tmp from earlier cmove. Not used again.
            },
            loc: GameLocation::Ae { id: 51449, offset: 0xe9 },
            sig: signature![
                0xf3, 0x0f, 0x11, 0x84, 0x24, 0xa0, 0x00, 0x00, 0x00,
                0x48, 0x85, 0xc9; 12
            ]
        },
        Descriptor::Patch {
            name: "BeginMaxChargeCalculation",
            enabled: || SETTINGS.general.enchanting_patch_en.get(),
            conflicts: None,
            hook: Hook::Call12 {
                entry: max_charge_begin_wrapper_se as *const u8,
                clobber: Register::Rax // Tmp from earlier cmove. Not used again.
            },
            loc: GameLocation::Se { id: 50557, offset: 0xe8 },
            sig: signature![
                0xf3, 0x0f, 0x11, 0x84, 0x24, 0xc0, 0x00, 0x00, 0x00,
                0x48, 0x85, 0xc9; 12
            ]
        },
        Descriptor::Patch {
            name: "EndMaxChargeCalculation",
            enabled: || SETTINGS.general.enchanting_patch_en.get(),
            conflicts: None,
            hook: Hook::Call12 {
                entry: max_charge_end_wrapper_ae as *const u8,
                clobber: Register::Rcx // Patch follows a function call.
            },
            loc: GameLocation::Ae { id: 51449, offset: 0x179 },
            sig: signature![
                0xf3, 0x0f, 0x10, 0x84, 0x24, 0xa0, 0x00, 0x00, 0x00,
                0xf3, 0x41, 0x0f, 0x5f, 0xc1; 14
            ]
        },
        Descriptor::Patch {
            name: "EndMaxChargeCalculation",
            enabled: || SETTINGS.general.enchanting_patch_en.get(),
            conflicts: None,
            hook: Hook::Call12 {
                entry: max_charge_end_wrapper_se as *const u8,
                clobber: Register::Rcx // Patch follows a function call.
            },
            loc: GameLocation::Se { id: 50557, offset: 0x207 },
            sig: signature![
                0xf3, 0x0f, 0x10, 0x84, 0x24, 0xc0, 0x00, 0x00, 0x00,
                0x41, 0x0f, 0x2f, 0xc0,
                0x77, 0x04,
                0x41, 0x0f, 0x28, 0xc0; 19
            ]
        },

        //
        // Replaces the original charge point calculation function call with a call
        // to our modified implementation, which caps the enchant level at 199.
        //
        // This fixes an issue with the games original equation for level values above 199.
        //
        // Note that we also replace the player_avo_get_current() call, so that we
        // can enforce a different formula cap for enchanting charge and magnitude.
        //
        Descriptor::Patch {
            name: "CalculateChargePointsPerUse",
            enabled: || SETTINGS.general.enchanting_patch_en.get(),
            conflicts: None,
            hook: Hook::Call12 {
                entry: calculate_charge_points_per_use_wrapper_ae as *const u8,
                clobber: Register::Rax
            },
            loc: GameLocation::Ae { id: 51449, offset: 0x314 },
            sig: signature![
                0x48, 0x8b, 0x0d, ?, ?, ?, ?,
                0x48, 0x81, 0xc1, ?, 0x00, 0x00, 0x00,
                0x48, 0x8b, 0x01,
                0xba, 0x17, 0x00, 0x00, 0x00,
                0xff, 0x50, 0x08,
                0x0f, 0x28, 0xc8,
                0x0f, 0x28, 0xc7,
                0xe8, ?, ?, ?, ?; 36
            ]
        },
        Descriptor::Patch {
            name: "CalculateChargePointsPerUse",
            enabled: || SETTINGS.general.enchanting_patch_en.get(),
            conflicts: None,
            hook: Hook::Call12 {
                entry: calculate_charge_points_per_use_wrapper_se as *const u8,
                clobber: Register::Rax
            },
            loc: GameLocation::Se { id: 50557, offset: 0x344, },
            sig: signature![
                0x48, 0x8b, 0x0d, ?, ?, ?, ?,
                0x48, 0x81, 0xc1, ?, 0x00, 0x00, 0x00,
                0x48, 0x8b, 0x01,
                0xba, 0x17, 0x00, 0x00, 0x00,
                0xff, 0x50, 0x08,
                0x0f, 0x28, 0xc8,
                0x0f, 0x28, 0xc7,
                0xe8, ?, ?, ?, ?; 36
            ]
        },

        //
        // Caps the effective skill level in calculations by always returning a damaged result.
        //
        Descriptor::Patch {
            name: "PlayerAVOGetCurrent",
            enabled: || SETTINGS.general.skill_formula_caps_en.get(),
            conflicts: None,
            hook: Hook::Jump12 {
                entry: player_avo_get_current_wrapper as *const u8,
                clobber: Register::Rax,
                trampoline: PLAYER_AVO_GET_CURRENT_RETURN_TRAMPOLINE.inner()
            },
            loc: GameLocation::Ae { id: 38462, offset: 0 },
            sig: signature![
                0x4c, 0x8b, 0xdc, 0x55, 0x56, 0x57, 0x41, 0x56,
                0x41, 0x57, 0x48, 0x83, 0xec, 0x50; 14
            ]
        },
        Descriptor::Patch {
            name: "PlayerAVOGetCurrent",
            enabled: || SETTINGS.general.skill_formula_caps_en.get(),
            conflicts: None,
            hook: Hook::Jump12 {
                entry: player_avo_get_current_wrapper as *const u8,
                clobber: Register::Rax,
                trampoline: PLAYER_AVO_GET_CURRENT_RETURN_TRAMPOLINE.inner()
            },
            loc: GameLocation::Se { id: 37517, offset: 0 },
            sig: signature![
                0x40, 0x55, 0x56, 0x57, 0x41, 0x56, 0x41, 0x57,
                0x48, 0x83, 0xec, 0x40; 12
            ]
        },

        //
        // Uncaps the formula for magic CDR, which is now capped by PlayerAVOGetCurrent(). Without
        // these patches, magic CDR cannot use skill levels above 100.
        //
        Descriptor::Patch {
            name: "CapMagickaCDR",
            enabled: || SETTINGS.general.skill_formula_caps_en.get(),
            conflicts: None,
            hook: Hook::None,
            loc: GameLocation::Ae { id: 27284, offset: 0x2c },
            sig: signature![0xf3, 0x0f, 0x5d, 0x0d, ?, ?, ?, ?; 8] // minss <100.0>, %xmm1
        },
        Descriptor::Patch {
            name: "CapMagickaCDR",
            enabled: || SETTINGS.general.skill_formula_caps_en.get(),
            conflicts: None,
            hook: Hook::None,
            loc: GameLocation::Se { id: 26616, offset: 0x34 },
            sig: signature![0x73, 0x44; 2] // jae <end of function>
        },

        //
        // Wraps the call to UpdateSkillList() to temporarily disable the formula cap patch, which
        // avoids a UI bug that would otherwise display the wrong level.
        //
        // We must implement this as a function wrapper, since our previous patch location
        // conflicted with Custom Skills Framework.
        //
        // This patch doesn't serve a real purpose other than to avoid confusing players.
        //
        Descriptor::Patch {
            name: "UpdateSkillList",
            enabled: || SETTINGS.general.skill_formula_caps_en.get() &&
                        SETTINGS.general.skill_formula_ui_fix_en.get(),
            conflicts: None,
            hook: Hook::Jump12 {
                entry: update_skill_list_wrapper as *const u8,
                clobber: Register::Rax, // Start of a function call.
                trampoline: UPDATE_SKILL_LIST_RETURN_TRAMPOLINE.inner()
            },
            loc: GameLocation::All {
                id_ae: 52525,
                id_se: 51652,
                offset_ae: 0,
                offset_se: 0
            },
            sig: signature![
                0x48, 0x8b, 0xc4, // mov %rsp, %rax
                0x55,             // push %rbp
                0x53,             // push %rbx
                0x56,             // push %rsi
                0x57,             // push %rdi
                0x41, 0x54,       // push %r12
                0x41, 0x55,       // push %r13
                0x41, 0x56; 13    // push %r14
            ]
        },

        // Applies the multipliers from the INI file to skill experience.
        Descriptor::Patch {
            name: "ImprovePlayerSkillPoints",
            enabled: || SETTINGS.general.skill_exp_mults_en.get(),
            conflicts: None,
            hook: Hook::Call12 {
                entry: improve_player_skill_points_wrapper_ae as *const u8,
                clobber: Register::Rcx // Written to after this patch.
            },
            loc: GameLocation::Ae { id: 41561, offset: 0xf1 },
            sig: signature![
                0xf3, 0x0f, 0x10, 0x44, 0x24, 0x30,
                0xf3, 0x0f, 0x59, 0xc6,
                0xf3, 0x0f, 0x58, 0x44, 0x24, 0x34; 16
            ]
        },
        Descriptor::Patch {
            name: "ImprovePlayerSkillPoints",
            enabled: || SETTINGS.general.skill_exp_mults_en.get(),
            conflicts: None,
            hook: Hook::Call12 {
                entry: improve_player_skill_points_wrapper_se as *const u8,
                clobber: Register::Rcx // Written to after this patch, garbage before patch.
            },
            loc: GameLocation::Se { id: 40554, offset: 0xdc },
            sig: signature![
                0xf3, 0x0f, 0x10, 0x44, 0x24, 0x30,
                0xf3, 0x0f, 0x59, 0xc7,
                0xf3, 0x0f, 0x58, 0x44, 0x24, 0x34; 16
            ]
        },

        //
        // Modifies the number of perk points obtained after the game has performed its
        // original calculation.
        //
        Descriptor::Patch {
            name: "ModifyPerkPool",
            enabled: || SETTINGS.general.perk_points_en.get(),
            conflicts: None,
            hook: Hook::Call12 {
                entry: modify_perk_pool_wrapper_ae as *const u8,
                clobber: Register::Rax
            },
            loc: GameLocation::Ae { id: 52538, offset: 0x62 },
            sig: signature![
                0x48, 0x8b, 0x15, ?, ?, ?, ?,
                0x0f, 0xb6, 0x8a, ?, 0x0b, 0x00, 0x00,
                0x8b, 0xc1,
                0x03, 0xc7,
                0x78, 0x09,
                0x40, 0x02, 0xcf,
                0x88, 0x8a, ?, 0x0b, 0x00, 0x00; 29
            ]
        },
        Descriptor::Patch {
            name: "ModifyPerkPool",
            enabled: || SETTINGS.general.perk_points_en.get(),
            conflicts: None,
            hook: Hook::Call12 {
                entry: modify_perk_pool_wrapper_se as *const u8,
                clobber: Register::Rax
            },
            loc: GameLocation::Se { id: 51665, offset: 0x8f },
            sig: signature![
                0x48, 0x8b, 0x15, ?, ?, ?, ?,
                0x0f, 0xb6, 0x8a, ?, 0x0b, 0x00, 0x00,
                0x8b, 0xc1,
                0x03, 0xc3,
                0x78, 0x08,
                0x02, 0xcb,
                0x88, 0x8a, ?, 0x0b, 0x00, 0x00; 28
            ]
        },

        //
        // Passes the EXP gain original calculated by the game to our hook for further
        // modification.
        //
        Descriptor::Patch {
            name: "ImproveLevelExpBySkillLevel",
            enabled: || SETTINGS.general.level_exp_mults_en.get(),
            conflicts: Some(LEVEL_MULT_CONFLICTS),
            hook: Hook::Call12 {
                entry: improve_level_exp_by_skill_level_wrapper_ae as *const u8,
                clobber: Register::Rdx // Will be smashed after this hook anyway.
            },
            loc: GameLocation::Ae { id: 41561, offset: 0x2cb },
            sig: signature![
                0xf3, 0x0f, 0x5c, 0xca,
                0xf3, 0x0f, 0x59, 0x0d, ?, ?, ?, ?; 12
            ]
        },
        Descriptor::Patch {
            name: "ImproveLevelExpBySkillLevel",
            enabled: || SETTINGS.general.level_exp_mults_en.get(),
            conflicts: Some(LEVEL_MULT_CONFLICTS),
            hook: Hook::Call12 {
                entry: improve_level_exp_by_skill_level_wrapper_se as *const u8,
                clobber: Register::Rax // Smashed earlier in the function.
            },
            loc: GameLocation::Se { id: 40576, offset: 0x70 },
            sig: signature![
                0xf3, 0x0f, 0x5c, 0xc1,
                0xf3, 0x0f, 0x59, 0x05, ?, ?, ?, ?; 12
            ]
        },

        //
        // Overwrites the attribute level-up function to adjust the gains based on the players
        // attribute selection.
        //
        // We inject this patch just after the player has made their attribute selection, and
        // replace what would have been a call to player_avo->ModBase(...). Then, we manually
        // invoke ModBase and ModCurrent for the attributes and carry weight specified in the INI
        // file.
        //
        // Note that this patch overwrites the carry weight change done in the games code as well.
        // It also means the game settings which would usually control these attributes are
        // ignored.
        //
        Descriptor::Patch {
            name: "ImproveAttributeWhenLevelUp",
            enabled: || SETTINGS.general.attr_points_en.get(),
            conflicts: None,
            hook: Hook::Call12 {
                entry: improve_attribute_when_level_up_wrapper as *const u8,
                clobber: Register::Rax
            },
            loc: GameLocation::All {
                id_se: 51037,
                id_ae: 51917,
                offset_se: 0x93,
                offset_ae: 0x93
            },
            sig: signature![
                0xff, 0x50, 0x28,
                0x83, 0x7f, 0x18, 0x1a,
                0x75, 0x22,
                0x48, 0x8b, 0x0d, ?, ?, ?, ?,
                0x48, 0x81, 0xc1, ?, 0x00, 0x00, 0x00,
                0x48, 0x8b, 0x01,
                0xf3, 0x0f, 0x10, 0x1d, ?, ?, ?, ?,
                0x33, 0xd2,
                0x44, 0x8d, 0x42, 0x20,
                0xff, 0x50, 0x30; 0x2b
            ]
        },

        //
        // Alters the reset level of legendarying a skill, and overwrites a check
        // which prevents the level from changing if its below 100.
        //
        Descriptor::Patch {
            name: "LegendaryResetSkillLevel",
            enabled: || SETTINGS.general.legendary_en.get(),
            conflicts: Some(LEGENDARY_CONFLICTS),
            hook: Hook::Call12 {
                entry: legendary_reset_skill_level_wrapper as *const u8,
                clobber: Register::Rax
            },
            loc: GameLocation::All {
                id_se: 51714,
                id_ae: 52591,
                offset_se: 0x236,
                offset_ae: 0x1d0
            },
            sig: signature![
                0x0f, 0x2f, 0x05, ?, ?, ?, ?,
                0x0f, 0x82, ?, ?, 0x00, 0x00,
                0x48, 0x8b, 0x0d, ?, ?, ?, ?,
                0x48, 0x81, 0xc1, ?, 0x00, 0x00, 0x00,
                0x48, 0x8b, 0x01,
                0xf3, 0x0f, 0x10, 0x15, ?, ?, ?, ?; 38
            ]
        },

        // Replaces the call to the legendary condition check function with our own.
        Descriptor::Patch {
            name: "CheckConditionForLegendarySkill",
            enabled: || SETTINGS.general.legendary_en.get(),
            conflicts: Some(LEGENDARY_CONFLICTS),
            hook: Hook::Call12 {
                entry: check_condition_for_legendary_skill_wrapper as *const u8,
                clobber: Register::Rdx
            },
            loc: GameLocation::All {
                id_se: 51647,
                id_ae: 52520,
                offset_se: 0x155,
                offset_ae: 0x14e
            },
            sig: signature![
                0x8b, 0xd0,
                0x48, 0x8d, 0x8f, ?, 0x00, 0x00, 0x00,
                0xff, 0x53, 0x18; 12
            ]
        },

        // As above, except this is for the function where the jump key is remapped.
        Descriptor::Patch {
            name: "CheckConditionForLegendarySkillAlt",
            enabled: || SETTINGS.general.legendary_en.get(),
            conflicts: Some(LEGENDARY_CONFLICTS),
            hook: Hook::Call12 {
                entry: check_condition_for_legendary_skill_wrapper as *const u8,
                clobber: Register::Rdx
            },
            loc: GameLocation::All {
                id_se: 51638,
                id_ae: 52510,
                offset_se: 0x4cc,
                offset_ae: 0x4d5
            },
            sig: signature![
                0x8b, 0xd0,
                0x48, 0x8d, 0x8f, ?, 0x00, 0x00, 0x00,
                0xff, 0x53, 0x18; 12
            ]
        },

        // As above, except this is for the UI button display.
        Descriptor::Patch {
            name: "HideLegendaryButton",
            enabled: || SETTINGS.general.legendary_en.get(),
            conflicts: Some(LEGENDARY_CONFLICTS),
            hook: Hook::Call12 {
                entry: hide_legendary_button_wrapper_ae as *const u8,
                clobber: Register::Rax
            },
            loc: GameLocation::Ae { id: 52527, offset: 0x153 },
            sig: signature![
                0x48, 0x8b, 0x0d, ?, ?, ?, ?,
                0x48, 0x81, 0xc1, ?, 0x00, 0x00, 0x00,
                0x48, 0x8b, 0x01,
                0x41, 0x8b, 0xd7,
                0xff, 0x50, 0x18; 23
            ]
        },
        Descriptor::Patch {
            name: "HideLegendaryButton",
            enabled: || SETTINGS.general.legendary_en.get(),
            conflicts: Some(LEGENDARY_CONFLICTS),
            hook: Hook::Call12 {
                entry: hide_legendary_button_wrapper_se as *const u8,
                clobber: Register::Rax
            },
            loc: GameLocation::Se { id: 51654, offset: 0x146 },
            sig: signature![
                0x48, 0x8b, 0x0d, ?, ?, ?, ?,
                0x48, 0x81, 0xc1, ?, 0x00, 0x00, 0x00,
                0x48, 0x8b, 0x01,
                0x8b, 0xd6,
                0xff, 0x50, 0x18; 22
            ]
        },

        //
        // Clears the legendary skill button when the player changes the skill they are hovering
        // over if the new skill is not a high enough level.
        //
        // This patch is a bit odd, because what is actually happening is that the game is
        // calling a scaleform API function to update the description under the perk tree
        // and that description takes in a skill level which is used, as far as I can tell,
        // only to determine if the legendary skill button should be shown or not.
        //
        // To be minimally invasive, we patch it so that we pass through the value except in
        // cases where our legendary button state conflicts with the games understood state,
        // in which case we pass either 100 or 99, whichever will get the button to
        // display correctly.
        //
        // Note that we also fix an engine bug where this function got the current skill value
        // instead of the base value.
        //
        Descriptor::Patch {
            name: "ClearLegendaryButton",
            enabled: || SETTINGS.general.legendary_en.get(),
            conflicts: Some(LEGENDARY_CONFLICTS),
            hook: Hook::Call12 {
                entry: clear_legendary_button_wrapper_ae as *const u8,
                clobber: Register::Rax
            },
            loc: GameLocation::Ae { id: 52527, offset: 0x16dd },
            sig: signature![
                0x48, 0x8b, 0x0d, ?, ?, ?, ?,
                0x48, 0x81, 0xc1, ?, 0x00, 0x00, 0x00,
                0x48, 0x8b, 0x01,
                0x41, 0x8b, 0xd7,
                0xff, 0x50, 0x08; 23
            ]
        },
        Descriptor::Patch {
            name: "ClearLegendaryButton",
            enabled: || SETTINGS.general.legendary_en.get(),
            conflicts: Some(LEGENDARY_CONFLICTS),
            hook: Hook::Call12 {
                entry: clear_legendary_button_wrapper_se as *const u8,
                clobber: Register::Rax
            },
            loc: GameLocation::Se { id: 51654, offset: 0x1621 },
            sig: signature![
                0x48, 0x8b, 0x0d, ?, ?, ?, ?,
                0x48, 0x81, 0xc1, ?, 0x00, 0x00, 0x00,
                0x48, 0x8b, 0x01,
                0x8b, 0xd6,
                0xff, 0x50, 0x08; 22
            ]
        }
    ];
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Patch implementations
////////////////////////////////////////////////////////////////////////////////////////////////////

/// The base game threshold for legendarying a skill.
const BASE_LEGENDARY_THRESHOLD: f32 = 100.0;

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Determines the real skill cap of the given skill.
extern "system" fn get_skill_cap_hook(
    skill: c_int
) -> f32 {
    assert!(SETTINGS.general.skill_caps_en.get());
    SETTINGS.skill_caps.get(ActorAttribute::from_raw_skill(skill).unwrap()).get() as f32
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Used to ensure that the max_charge critical section is not entered twice.
static IS_USING_CHARGE_CAP: AtomicBool = AtomicBool::new(false);

/// Begins a calculation for weapon charge by setting the enchant cap to use the charge value.
extern "system" fn max_charge_begin_hook(
    enchant_type: u32
) {
    const WEAPON_ENCHANT_TYPE: u32 = 0x29; // Defined by the game.
    if enchant_type == WEAPON_ENCHANT_TYPE {
        assert!(!IS_USING_CHARGE_CAP.swap(true, Ordering::Relaxed));
    }
}

/// Ends a calculation for weapon charge by returning the cap mode to magnitude, if necessary.
extern "system" fn max_charge_end_hook() {
    let _ = IS_USING_CHARGE_CAP.compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed);
}

///
/// Reimplements the enchantment charge point equation.
///
/// The original equation would fall apart for levels above 199, so this
/// implementation caps the level in the calculation to 199.
///
extern "system" fn calculate_charge_points_per_use_hook(
    base_points: f32,
    max_charge: f32
) -> f32 {
    assert!(SETTINGS.general.enchanting_patch_en.get());

    let cost_exponent = *ENCHANTING_COST_EXPONENT.get();
    let cost_base = *ENCHANTING_SKILL_COST_BASE.get();
    let cost_scale = *ENCHANTING_SKILL_COST_SCALE.get();
    let cost_mult = *ENCHANTING_SKILL_COST_MULT.get();
    let cap = (SETTINGS.enchant.charge_cap.get() as f32).min(199.0).min(
        SETTINGS.skill_formula_caps.get(ActorAttribute::Enchanting).get() as f32
    );
    let enchanting_level = cap.min(PlayerCharacter::get_current(ActorAttribute::Enchanting));

    let base = cost_mult * base_points.powf(cost_exponent);
    if SETTINGS.enchant.use_linear_charge.get() {
        // Linearly scale between current min/max of charge points. Max scales with skills/perks,
        // so this isn't perfectly linear. It still smooths the EQ a lot, though.
        let max_level_scale = (cap * cost_base).powf(cost_scale);
        let slope = (max_charge * max_level_scale) / (base * (1.0 - max_level_scale) * cap);
        let intercept = max_charge / base;
        let linear_charge = slope * enchanting_level + intercept;
        max_charge / linear_charge
    } else {
        // Original game equation.
        base * (1.0 - (enchanting_level * cost_base).powf(cost_scale))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Used to temporarily disable the formula cap during UI functions.
static IS_FORMULA_CAP_DISABLED_FOR_UI: AtomicBool = AtomicBool::new(false);

/// Caps the formula results for each skill.
extern "system" fn player_avo_get_current_hook(
    av: *mut ActorValueOwner,
    attr: c_int
) -> f32 {
    assert!(SETTINGS.general.skill_formula_caps_en.get());

    let mut val = unsafe {
        // SAFETY: We are passing through the original arguments.
        avo_get_current_unchecked(av, attr)
    };

    // If we're in a UI function, don't apply the cap. Also, ignore NPCs.
    if IS_FORMULA_CAP_DISABLED_FOR_UI.load(Ordering::Relaxed) ||
            av != PlayerCharacter::get_avo() {
        return val;
    }

    if let Ok(skill) = ActorAttribute::from_raw_skill(attr) {
        let mut cap = SETTINGS.skill_formula_caps.get(skill).get() as f32;

        // Enforce the additional enchanting caps.
        if skill == ActorAttribute::Enchanting {
            cap = cap.min(if IS_USING_CHARGE_CAP.load(Ordering::Relaxed) {
                SETTINGS.enchant.charge_cap.get() as f32
            } else {
                SETTINGS.enchant.magnitude_cap.get() as f32
            });
        }

        val = val.min(cap).max(0.0);
    }

    return val;
}

/// Wraps the function which determines the number displayed for the skill level in the skills menu.
///
/// Without this patch, the skill level will appear to be capped at the formula cap. The color will
/// also appear to be damaged/not fortified depending on how the true value differs from the cap.
///
/// Difficult to prove it with this function, but it appears to return void.
extern "system" fn update_skill_list_hook(
    unk: *mut ()
) {
    assert!(!IS_FORMULA_CAP_DISABLED_FOR_UI.swap(true, Ordering::Relaxed));
    unsafe { update_skill_list_unchecked(unk); }
    assert!(IS_FORMULA_CAP_DISABLED_FOR_UI.swap(false, Ordering::Relaxed));
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Applies a multiplier to the exp gain for the given skill.
extern "system" fn improve_player_skill_points_hook(
    attr: c_int,
    mut exp_base: f32,
    mut exp_offset: f32
) -> f32 {
    assert!(SETTINGS.general.skill_exp_mults_en.get());

    if let Ok(skill) = ActorAttribute::from_raw_skill(attr) {
        let base_mult = SETTINGS.skill_exp_mults.get(skill).get();
        let skill_mult = SETTINGS.skill_exp_mults_with_skills.get(skill).get_nearest(
            PlayerCharacter::get_base(skill) as u32
        );
        let pc_mult = SETTINGS.skill_exp_mults_with_pc_lvl.get(skill).get_nearest(
            PlayerCharacter::get_level()
        );

        exp_base   *= base_mult.base   * skill_mult.base   * pc_mult.base;
        exp_offset *= base_mult.offset * skill_mult.offset * pc_mult.offset;
    }

    exp_base + exp_offset
}

/// Multiplies the exp gain of a level-up by the configured multiplier.
extern "system" fn improve_level_exp_by_skill_level_hook(
    mut exp: f32,
    attr: c_int
) -> f32 {
    assert!(SETTINGS.general.level_exp_mults_en.get());

    if let Ok(skill) = ActorAttribute::from_raw_skill(attr) {
        exp *= SETTINGS.level_exp_mults.get(skill).get()
             * SETTINGS.level_exp_mults_with_skills.get(skill)
                       .get_nearest(PlayerCharacter::get_base(skill) as u32)
             * SETTINGS.level_exp_mults_with_pc_lvl.get(skill)
                       .get_nearest(PlayerCharacter::get_level());
    }

    exp * *XP_PER_SKILL_RANK.get()
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Adjusts the number of perks the player recieves at level-up.
extern "system" fn modify_perk_pool_hook(
    count: i8
) {
    assert!(SETTINGS.general.perk_points_en.get());

    let pool = PlayerCharacter::get_perk_pool();
    let delta = std::cmp::min(
        0xFF,
        SETTINGS.perks_at_lvl_up.get_cumulative_delta(PlayerCharacter::get_level())
    );
    let res = (pool.get() as i16) + (if count > 0 { delta as i16 } else { count as i16 });
    pool.set(std::cmp::max(0, std::cmp::min(0xff, res)) as u8);
}

///
/// Adjusts the attribute gain at each level-up based on the configured settings.
///
extern "system" fn improve_attribute_when_level_up_hook(
    choice: c_int
) {
    assert!(SETTINGS.general.attr_points_en.get());

    let player_level = PlayerCharacter::get_level();
    let (hp, mp, sp, cw) = match ActorAttribute::from_raw(choice).unwrap() {
        ActorAttribute::Health => (
            SETTINGS.hp_at_lvl_up.get_nearest(player_level) as f32,
            SETTINGS.mp_at_hp_lvl_up.get_nearest(player_level) as f32,
            SETTINGS.sp_at_hp_lvl_up.get_nearest(player_level) as f32,
            SETTINGS.cw_at_hp_lvl_up.get_nearest(player_level) as f32
        ),
        ActorAttribute::Magicka => (
            SETTINGS.hp_at_mp_lvl_up.get_nearest(player_level) as f32,
            SETTINGS.mp_at_lvl_up.get_nearest(player_level) as f32,
            SETTINGS.sp_at_mp_lvl_up.get_nearest(player_level) as f32,
            SETTINGS.cw_at_mp_lvl_up.get_nearest(player_level) as f32
        ),
        ActorAttribute::Stamina => (
            SETTINGS.hp_at_sp_lvl_up.get_nearest(player_level) as f32,
            SETTINGS.mp_at_sp_lvl_up.get_nearest(player_level) as f32,
            SETTINGS.sp_at_lvl_up.get_nearest(player_level) as f32,
            SETTINGS.cw_at_sp_lvl_up.get_nearest(player_level) as f32
        ),
        _ => panic!("Cannot get the attribute level up with an invalid choice.")
    };

    PlayerCharacter::mod_base(ActorAttribute::Health, hp);
    PlayerCharacter::mod_base(ActorAttribute::Magicka, mp);
    PlayerCharacter::mod_base(ActorAttribute::Stamina, sp);
    PlayerCharacter::mod_current(ActorAttribute::CarryWeight, cw);
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Determines what level a skill should take on after being legendary'd.
extern "system" fn legendary_reset_skill_level_hook(
    base_level: f32
) -> f32 {
    assert!(SETTINGS.general.legendary_en.get());
    assert!(base_level >= 0.0);
    let base_val = *LEGENDARY_SKILL_RESET_VALUE.get();

    // Check if legendarying should reset the level at all.
    if SETTINGS.legendary.keep_skill_level.get() {
        return base_level;
    }

    // 0 in the conf file means we should use the default value.
    let mut reset_level = SETTINGS.legendary.skill_level_after.get() as f32;
    if reset_level == 0.0 {
        reset_level = base_val;
    }

    // Don't allow legendarying to raise the skill level.
    reset_level.min(base_level)
}

///
/// Overwrites the check which determines when a skill can be legendary'd.
///
/// Due to how this function is injected, we return a "bool" based on the legendary
/// threshold. If the condition is valid, we return the threshold. Otherwise, we
/// return threshold - 1.
///
extern "system" fn check_condition_for_legendary_skill_hook(
    skill: c_int
) -> f32 {
    assert!(SETTINGS.general.legendary_en.get());
    let skill = ActorAttribute::from_raw_skill(skill).unwrap();

    if PlayerCharacter::get_base(skill) as u32 >= SETTINGS.legendary.skill_level_en.get() {
        BASE_LEGENDARY_THRESHOLD
    } else {
        BASE_LEGENDARY_THRESHOLD - 1.0
    }
}

///
/// Determines if the legendary button should be displayed for the given skill.
///
/// Due to how this function is injected, we return a "bool" based on the legendary
/// threshold. If the condition is valid, we return the threshold. Otherwise, we
/// return threshold - 1.
///
extern "system" fn hide_legendary_button_hook(
    skill: c_int
) -> f32 {
    assert!(SETTINGS.general.legendary_en.get());
    let skill = ActorAttribute::from_raw_skill(skill).unwrap();

    if (PlayerCharacter::get_base(skill) as u32 >= SETTINGS.legendary.skill_level_en.get())
            && !SETTINGS.legendary.hide_button.get() {
        BASE_LEGENDARY_THRESHOLD
    } else {
        BASE_LEGENDARY_THRESHOLD - 1.0
    }
}

///
/// Determines if we should continue to display the legendary button after moving the skill view.
///
/// The value determined depends on how the state of the legendary button hint differs from
/// what we want. If it is in the correct state, we make no changes. Otherwise, we return the
/// threshold or the threshold - 1 depending on if we want the hint to be invisible or
/// visible, respectively.
///
extern "system" fn clear_legendary_button_hook(
    skill: c_int
) -> f32 {
    assert!(SETTINGS.general.legendary_en.get());

    if let Ok(skill) = ActorAttribute::from_raw_skill(skill) {
        let level = PlayerCharacter::get_base(skill);
        let game_vis = level >= BASE_LEGENDARY_THRESHOLD;
        let mod_vis = !SETTINGS.legendary.hide_button.get()
            && (PlayerCharacter::get_base(skill) as u32 >= SETTINGS.legendary.skill_level_en.get());

        if game_vis == mod_vis {
            level
        } else if game_vis { // visible, but shouldn't be.
            BASE_LEGENDARY_THRESHOLD - 1.0
        } else { // invisible, but shouldn't be.
            BASE_LEGENDARY_THRESHOLD
        }
    } else {
        // Some other perk menu. E.g. vampire or werewolf
        unsafe { PlayerCharacter::get_base_unchecked(skill) }
    }
}
