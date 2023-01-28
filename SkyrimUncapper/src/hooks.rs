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
use crate::hook_wrappers::*;
use crate::patcher::{Descriptor, Hook, HookFn, GameLocation, GameRef};
use crate::safe::signature;
use crate::skyrim::{ActorAttribute, ActorValueOwner, PlayerSkills};
use crate::skyrim::{player_avo_get_base, player_avo_get_current_original, game_setting};
use crate::skyrim::{player_avo_mod_base, player_avo_mod_current, get_player_level};

///
/// Trampolines used by hooks to return to game code.
///
/// Boing!
///
#[no_mangle] static improve_player_skill_points_return_trampoline: GameRef<usize> = GameRef::new();
#[no_mangle] static modify_perk_pool_return_trampoline: GameRef<usize> = GameRef::new();
#[no_mangle] static player_avo_get_current_return_trampoline: GameRef<usize> = GameRef::new();

disarray::disarray! {
    /// The hooks which must be installed by the game patcher.
    pub static HOOK_SIGNATURES: [Descriptor; NUM_HOOK_SIGNATURES] = [
        //
        // Injects the code which alters the real skill cap of each skill.
        //
        // Note that the last two bytes of this patch must be overwritten with NOPs
        // and returned to, at the request of the author of the eXPerience mod (17751).
        // This is handled by the patcher, we need only make our signature long enough.
        //
        Descriptor::Patch {
            name: "GetSkillCap",
            enabled: settings::is_skill_cap_enabled,
            hook: Hook::Call6(unsafe { HookFn::new(skill_cap_patch_wrapper as *const u8) }),
            loc: GameLocation::Id { id: 41561, offset: 0x76 },
            sig: signature![0xf3, 0x44, 0x0f, 0x10, 0x15, ?, ?, ?, ?],
            trampoline: None
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
            enabled: settings::is_enchant_patch_enabled,
            hook: Hook::Call6(unsafe { HookFn::new(calculate_charge_points_per_use_wrapper as *const u8) }),
            loc: GameLocation::Id { id: 51449, offset: 0x32a },
            sig: signature![
                0xff, 0x50, 0x08, 0x0f, 0x28, 0xc8, 0x0f, 0x28,
                0xc7, 0xe8, 0x08, 0x6f, 0xb2, 0xff
            ],
            trampoline: None
        },

        //
        // Caps the effective skill level in calculations by always returning a damaged result.
        //
        Descriptor::Patch {
            name: "PlayerAVOGetCurrent",
            enabled: settings::is_skill_formula_cap_enabled,
            hook: Hook::Jump6(unsafe { HookFn::new(player_avo_get_current_hook as *const u8) }),
            loc: GameLocation::Id { id: 38462, offset: 0 },
            sig: signature![0x4c, 0x8b, 0xdc, 0x55, 0x56, 0x57],
            trampoline: Some(player_avo_get_current_return_trampoline.inner())
        },

        //
        // Overwrites the skill display player_avo_get_current() call to display the
        // actual, non-damaged, skill level.
        //
        // This patch doesn't serve a real purpose other than to avoid confusing players.
        //
        Descriptor::Patch {
            name: "DisplayTrueSkillLevel",
            enabled: settings::is_skill_formula_cap_enabled,
            hook: Hook::Call6(unsafe { HookFn::new(display_true_skill_level_hook as *const u8) }),
            loc: GameLocation::Id { id: 52525, offset: 0x120 },
            sig: signature![0xff, 0x50, 0x08, 0xf3, 0x0f, 0x2c, 0xc8],
            trampoline: None
        },

        //
        // Overwrites the skill color displays call to player_avo_get_current() to
        // call the original function.
        //
        // This patch exists for the same reason as the above patch.
        //
        Descriptor::Patch {
            name: "DisplayTrueSkillColor",
            enabled: settings::is_skill_formula_cap_enabled,
            hook: Hook::Call6(unsafe { HookFn::new(display_true_skill_color_hook as *const u8) }),
            loc: GameLocation::Id { id: 52945, offset: 0x32 },
            sig: signature![0xff, 0x50, 0x08, 0x48, 0x8b, 0x86, ?, 0x00, 0x00, 0x00],
            trampoline: None
        },

        // Prevents the skill training function from applying our multipliers.
        Descriptor::Patch {
            name: "ImproveSkillByTraining",
            enabled: settings::is_skill_exp_enabled,
            hook: Hook::Call5(unsafe { HookFn::new(improve_player_skill_points_original as *const u8) }),
            loc: GameLocation::Id { id: 41562, offset: 0x98 },
            sig: signature![0xe8, ?, ?, ?, ?],
            trampoline: None
        },

        // Applies the multipliers from the INI file to skill experience.
        Descriptor::Patch {
            name: "ImprovePlayerSkillPoints",
            enabled: settings::is_skill_exp_enabled,
            hook: Hook::Jump6(unsafe { HookFn::new(improve_player_skill_points_hook as *const u8) }),
            loc: GameLocation::Id { id: 41561, offset: 0 },
            sig: signature![0x48, 0x8b, 0xc4, 0x57, 0x41, 0x54],
            trampoline: Some(improve_player_skill_points_return_trampoline.inner())
        },

        //
        // Modifies the number of perk points obtained after the game has performed its
        // original calculation.
        //
        Descriptor::Patch {
            name: "ModifyPerkPool",
            enabled: settings::is_perk_points_enabled,
            hook: Hook::Jump6(unsafe { HookFn::new(modify_perk_pool_wrapper as *const u8) }),
            loc: GameLocation::Id { id: 52538, offset: 0x70 },
            sig: signature![0x8b, 0xc1, 0x03, 0xc7, 0x78, 0x09, 0x40, 0x02, 0xcf],
            trampoline: Some(modify_perk_pool_return_trampoline.inner())
        },

        //
        // Passes the EXP gain original calculated by the game to our hook for further
        // modification.
        //
        Descriptor::Patch {
            name: "ImproveLevelExpBySkillLevel",
            enabled: settings::is_level_exp_enabled,
            hook: Hook::Call6(unsafe { HookFn::new(improve_level_exp_by_skill_level_wrapper as *const u8) }),
            loc: GameLocation::Id { id: 41561, offset: 0x2d7 },
            sig: signature![0xf3, 0x0f, 0x58, 0x08, 0xf3, 0x0f, 0x11, 0x08],
            trampoline: None
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
            enabled: settings::is_attr_points_enabled,
            hook: Hook::Call6(unsafe { HookFn::new(improve_attribute_when_level_up_hook as *const u8) }),
            loc: GameLocation::Id { id: 51917, offset: 0x93 },
            sig: signature![
                0xff, 0x50, 0x28, 0x83, 0x7f, 0x18, 0x1a, 0x75,
                0x22, 0x48, 0x8b, 0x0d,    ?,    ?,    ?,    ?,
                0x48, 0x81, 0xc1,    ?, 0x00, 0x00, 0x00, 0x48,
                0x8b, 0x01, 0xf3, 0x0f, 0x10, 0x1d,    ?,    ?,
                   ?,    ?, 0x33, 0xd2, 0x44, 0x8d, 0x42, 0x20,
                0xff, 0x50, 0x30
            ],
            trampoline: None
        },

        // Alters the reset level of legendarying a skill.
        Descriptor::Patch {
            name: "LegendaryResetSkillLevel",
            enabled: settings::is_legendary_enabled,
            hook: Hook::Call6(unsafe { HookFn::new(legendary_reset_skill_level_wrapper as *const u8) }),
            loc: GameLocation::Id { id: 52591, offset: 0x1d7 },
            sig: signature![0x0f, 0x82, ?, ?, ?, ?],
            trampoline: None
        },

        // Replaces the call to the legendary condition check function with our own.
        Descriptor::Patch {
            name: "CheckConditionForLegendarySkill",
            enabled: settings::is_legendary_enabled,
            hook: Hook::Call6(unsafe { HookFn::new(check_condition_for_legendary_skill_wrapper as *const u8) }),
            loc: GameLocation::Id { id: 52520, offset: 0x157 },
            sig: signature![0xff, 0x53, 0x18, 0x0f, 0x2f, 0x05, ?, ?, ?, ?],
            trampoline: None
        },

        // As above, except this is for the function where the jump key is remapped.
        Descriptor::Patch {
            name: "CheckConditionForLegendarySkillAlt",
            enabled: settings::is_legendary_enabled,
            hook: Hook::Call6(unsafe { HookFn::new(check_condition_for_legendary_skill_wrapper as *const u8) }),
            loc: GameLocation::Id { id: 52510, offset: 0x4de },
            sig: signature![0xff, 0x53, 0x18, 0x0f, 0x2f, 0x05, ?, ?, ?, ?],
            trampoline: None
        },

        // As above, except this is for the UI button display.
        Descriptor::Patch {
            name: "HideLegendaryButton",
            enabled: settings::is_legendary_enabled,
            hook: Hook::Call6(unsafe { HookFn::new(hide_legendary_button_wrapper as *const u8) }),
            loc: GameLocation::Id { id: 52527, offset: 0x167 },
            sig: signature![0xff, 0x50, 0x18, 0x0f, 0x2f, 0x05, ?, ?, ?, ?],
            trampoline: None
        }
    ];
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
    skse_assert!(settings::is_skill_formula_cap_enabled());
    let attr = ActorAttribute::from_raw(attr).unwrap();

    let mut val = unsafe {
        // SAFETY: We are passing through the original arguments.
        player_avo_get_current_original(av, attr)
    };

    if attr.is_skill() {
        val = val.min(settings::get_skill_formula_cap(attr)).max(0.0);
    }

    return val;
}

/// Applies a multiplier to the exp gain for the given skill.
extern "system" fn improve_player_skill_points_hook(
    skill_data: *mut PlayerSkills,
    attr: c_int,
    mut exp: f32,
    unk1: u64,
    unk2: u32,
    unk3: u8,
    unk4: bool
) {
    skse_assert!(settings::is_skill_exp_enabled());
    let attr = ActorAttribute::from_raw(attr).unwrap();

    if attr.is_skill() {
        exp *= settings::get_skill_exp_mult(
            attr,
            player_avo_get_base(attr) as u32,
            get_player_level()
        );
    }

    unsafe {
        // SAFETY: We give it the same args, except the modified exp.
        improve_player_skill_points_original(skill_data, attr, exp, unk1, unk2, unk3, unk4);
    }
}

/// Adjusts the number of perks the player recieves at level-up.
#[no_mangle]
extern "system" fn modify_perk_pool_hook(
    points: u8,
    count: i8
) -> u8 {
    skse_assert!(settings::is_perk_points_enabled());
    let delta = std::cmp::min(0xFF, settings::get_perk_delta(get_player_level()));
    let res = (points as i16) + (if count > 0 { delta as i16 } else { count as i16 });
    std::cmp::max(0, std::cmp::min(0xff, res)) as u8
}

/// Multiplies the exp gain of a level-up by the configured multiplier.
#[no_mangle]
extern "system" fn improve_level_exp_by_skill_level_hook(
    mut exp: f32,
    attr: c_int
) -> f32 {
    skse_assert!(settings::is_level_exp_enabled());
    let attr = ActorAttribute::from_raw(attr).unwrap();

    if attr.is_skill() {
        exp *= settings::get_level_exp_mult(
            attr,
            player_avo_get_base(attr) as u32,
            get_player_level()
        );
    }

    exp
}

///
/// Adjusts the attribute gain at each level-up based on the configured settings.
///
/// This function overwrites a call to player_avo->mod_base(). Since we're overwriting
/// a call, we don't need to reg save and, thus, don't need a wrapper. We also overwrite
/// the carry weight level-up code.
///
extern "system" fn improve_attribute_when_level_up_hook(
    _av: *mut ActorValueOwner,
    choice: c_int
) {
    skse_assert!(settings::is_attr_points_enabled());
    let choice = ActorAttribute::from_raw(choice).unwrap();

    let (hp, mp, sp, cw) = settings::get_attribute_level_up(get_player_level(), choice);
    player_avo_mod_base(ActorAttribute::Health, hp);
    player_avo_mod_base(ActorAttribute::Magicka, mp);
    player_avo_mod_base(ActorAttribute::Stamina, sp);
    player_avo_mod_current(ActorAttribute::CarryWeight, cw);
}

/// Determines what level a skill should take on after being legendary'd.
#[no_mangle]
extern "system" fn legendary_reset_skill_level_hook(
    base_level: f32
) {
    skse_assert!(settings::is_legendary_enabled());
    skse_assert!(base_level >= 0.0);

    let reset_val = game_setting!("fLegendarySkillResetValue");
    reset_val.set_float(settings::get_post_legendary_skill_level(
            reset_val.get_float(),
            base_level
    ));
}

/// Overwrites the check which determines when a skill can be legendary'd.
#[no_mangle]
extern "system" fn check_condition_for_legendary_skill_hook(
    skill: c_int
) -> bool {
    skse_assert!(settings::is_legendary_enabled());
    let skill = ActorAttribute::from_raw(skill).unwrap();
    skse_assert!(skill.is_skill());
    settings::is_legendary_available(player_avo_get_base(skill) as u32)
}

/// Determines if the legendary button should be displayed for the given skill.
#[no_mangle]
extern "system" fn hide_legendary_button_hook(
    skill: c_int
) -> bool {
    skse_assert!(settings::is_legendary_enabled());
    let skill = ActorAttribute::from_raw(skill).unwrap();
    skse_assert!(skill.is_skill());
    settings::is_legendary_button_visible(player_avo_get_base(skill) as u32)
}
