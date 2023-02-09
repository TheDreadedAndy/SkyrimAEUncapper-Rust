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

use skyrim_patcher::{Descriptor, Hook, Register, GameLocation, AddrId, GameRef, signature};

use crate::settings;
use crate::hook_wrappers::*;
use crate::skyrim::*;

/// The base game threshold for legendarying a skill.
const BASE_LEGENDARY_THRESHOLD: f32 = 100.0;

//
// Trampolines used by hooks to return to game code.
//
// Boing!
//
#[no_mangle]
static improve_skill_by_training_return_trampoline: GameRef<usize> = GameRef::new();
#[no_mangle]
static improve_player_skill_points_return_trampoline: GameRef<usize> = GameRef::new();
#[no_mangle]
static player_avo_get_current_return_trampoline: GameRef<usize> = GameRef::new();

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
            hook: Hook::Call12 {
                entry: skill_cap_patch_wrapper as *const u8,
                clobber: Register::Rax
            },
            loc: GameLocation::Id { id: AddrId::All { se: 40554, ae: 41561 }, offset: 0x5e },
            sig: signature![
                0x48, 0x8b, 0x0d, ?, ?, ?, ?,
                0x48, 0x81, 0xc1, ?, 0x00, 0x00, 0x00,
                0x48, 0x8b, 0x01,
                0xff, 0x50, 0x18,
                0x44, 0x0f, 0x28, 0xc0,
                0xf3, 0x44, 0x0f, 0x10, 0x15, ?, ?, ?, ?; 33
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
            enabled: settings::is_enchant_patch_enabled,
            hook: Hook::Call12 {
                entry: max_charge_begin_wrapper as *const u8,
                clobber: Register::Rax // Tmp from earlier cmove. Not used again.
            },
            loc: GameLocation::Id { id: AddrId::All { se: 50557, ae: 51449 }, offset: 0xe9 },
            sig: signature![
                0xf3, 0x0f, 0x11, 0x84, 0x24, 0xa0, 0x00, 0x00, 0x00,
                0x48, 0x85, 0xc9; 12
            ]
        },
        Descriptor::Patch {
            name: "EndMaxChargeCalculation",
            enabled: settings::is_enchant_patch_enabled,
            hook: Hook::Call12 {
                entry: max_charge_end_wrapper as *const u8,
                clobber: Register::Rcx // Patch follows a function call.
            },
            loc: GameLocation::Id { id: AddrId::All { se: 50557, ae: 51449 }, offset: 0x179 },
            sig: signature![
                0xf3, 0x0f, 0x10, 0x84, 0x24, 0xa0, 0x00, 0x00, 0x00,
                0xf3, 0x41, 0x0f, 0x5f, 0xc1; 14
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
            enabled: settings::is_enchant_patch_enabled,
            hook: Hook::Call12 {
                entry: calculate_charge_points_per_use_wrapper as *const u8,
                clobber: Register::Rax
            },
            loc: GameLocation::Id { id: AddrId::All { se: 50557, ae: 51449 }, offset: 0x314 },
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
            enabled: settings::is_skill_formula_cap_enabled,
            hook: Hook::Jump12 {
                entry: player_avo_get_current_hook as *const u8,
                clobber: Register::Rax,
                trampoline: player_avo_get_current_return_trampoline.inner()
            },
            loc: GameLocation::Id { id: AddrId::All { se: 37517, ae: 38462 }, offset: 0 },
            sig: signature![
                0x4c, 0x8b, 0xdc, 0x55, 0x56, 0x57, 0x41, 0x56,
                0x41, 0x57, 0x48, 0x83, 0xec, 0x50; 14
            ]
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
            hook: Hook::Call12 {
                entry: display_true_skill_level_hook as *const u8,
                clobber: Register::Rax
            },
            loc: GameLocation::Id { id: AddrId::All { se: 51652, ae: 52525 }, offset: 0x10d },
            sig: signature![
                0x48, 0x8b, 0x0d, ?, ?, ?, ?,
                0x48, 0x81, 0xc1, ?, 0x00, 0x00, 0x00,
                0x48, 0x8b, 0x01,
                0x8b, 0xd7,
                0xff, 0x50, 0x08; 22
            ]
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
            hook: Hook::Call12 {
                entry: display_true_skill_color_hook as *const u8,
                clobber: Register::Rax
            },
            loc: GameLocation::Id { id: AddrId::All { se: 52059, ae: 52945 }, offset: 0x24 },
            sig: signature![
                0x48, 0x8b, 0x86, ?, 0x00, 0x00, 0x00,
                0x48, 0x8d, 0x8e, ?, 0x00, 0x00, 0x00,
                0xff, 0x50, 0x08; 17
            ]
        },

        // Prevents the skill training function from applying our multipliers.
        Descriptor::Patch {
            name: "ImproveSkillByTraining",
            enabled: settings::is_skill_exp_enabled,
            hook: Hook::Jump12 {
                entry: improve_skill_by_training_hook as *const u8,
                clobber: Register::Rax,
                trampoline: improve_skill_by_training_return_trampoline.inner()
            },
            loc: GameLocation::Id { id: AddrId::All { se: 40555, ae: 41562 }, offset: 0x90 },
            sig: signature![
                0x49, 0x8b, 0xcf,
                0x44, 0x89, 0x6c, 0x24, 0x20,
                0xe8, ?, ?, ?, ?; 13
            ]
        },

        // Applies the multipliers from the INI file to skill experience.
        Descriptor::Patch {
            name: "ImprovePlayerSkillPoints",
            enabled: settings::is_skill_exp_enabled,
            hook: Hook::Jump12 {
                entry: improve_player_skill_points_hook as *const u8,
                clobber: Register::Rax,
                trampoline: improve_player_skill_points_return_trampoline.inner()
            },
            loc: GameLocation::Id { id: AddrId::All { se: 40554, ae: 41561 }, offset: 0 },
            sig: signature![
                0x48, 0x8b, 0xc4,
                0x57,
                0x41, 0x54,
                0x41, 0x55,
                0x41, 0x56,
                0x41, 0x57; 12
            ]
        },

        //
        // Modifies the number of perk points obtained after the game has performed its
        // original calculation.
        //
        Descriptor::Patch {
            name: "ModifyPerkPool",
            enabled: settings::is_perk_points_enabled,
            hook: Hook::Call12 {
                entry: modify_perk_pool_wrapper as *const u8,
                clobber: Register::Rax
            },
            loc: GameLocation::Id { id: AddrId::All { se: 51665, ae: 52538 }, offset: 0x62 },
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

        //
        // Passes the EXP gain original calculated by the game to our hook for further
        // modification.
        //
        Descriptor::Patch {
            name: "ImproveLevelExpBySkillLevel",
            enabled: settings::is_level_exp_enabled,
            hook: Hook::Call12 {
                entry: improve_level_exp_by_skill_level_wrapper as *const u8,
                clobber: Register::Rcx
            },
            loc: GameLocation::Id { id: AddrId::All { se: 40554, ae: 41561 }, offset: 0x2d7 },
            sig: signature![
                0xf3, 0x0f, 0x58, 0x08,
                0xf3, 0x0f, 0x11, 0x08,
                0x8b, 0xd6,
                0x48, 0x8b, 0x0d, ?, ?, ?, ?; 17
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
            enabled: settings::is_attr_points_enabled,
            hook: Hook::Call12 {
                entry: improve_attribute_when_level_up_wrapper as *const u8,
                clobber: Register::Rax
            },
            loc: GameLocation::Id { id: AddrId::All { se: 51037, ae: 51917 }, offset: 0x93 },
            sig: signature![
                0xff, 0x50, 0x28, 0x83, 0x7f, 0x18, 0x1a, 0x75,
                0x22, 0x48, 0x8b, 0x0d,    ?,    ?,    ?,    ?,
                0x48, 0x81, 0xc1,    ?, 0x00, 0x00, 0x00, 0x48,
                0x8b, 0x01, 0xf3, 0x0f, 0x10, 0x1d,    ?,    ?,
                   ?,    ?, 0x33, 0xd2, 0x44, 0x8d, 0x42, 0x20,
                0xff, 0x50, 0x30; 0x2b
            ]
        },

        //
        // Alters the reset level of legendarying a skill, and overwrites a check
        // which prevents the level from changing if its below 100.
        //
        Descriptor::Patch {
            name: "LegendaryResetSkillLevel",
            enabled: settings::is_legendary_enabled,
            hook: Hook::Call12 {
                entry: legendary_reset_skill_level_wrapper as *const u8,
                clobber: Register::Rax
            },
            loc: GameLocation::Id { id: AddrId::All { se: 51714, ae: 52591 }, offset: 0x1d0 },
            sig: signature![
                0x0f, 0x2f, 0x05, ?, ?, ?, ?,
                0x0f, 0x82, 0x27, 0x01, 0x00, 0x00,
                0x48, 0x8b, 0x0d, ?, ?, ?, ?,
                0x48, 0x81, 0xc1, ?, 0x00, 0x00, 0x00,
                0x48, 0x8b, 0x01,
                0xf3, 0x0f, 0x10, 0x15, ?, ?, ?, ?; 38
            ]
        },

        // Replaces the call to the legendary condition check function with our own.
        Descriptor::Patch {
            name: "CheckConditionForLegendarySkill",
            enabled: settings::is_legendary_enabled,
            hook: Hook::Call12 {
                entry: check_condition_for_legendary_skill_wrapper as *const u8,
                clobber: Register::Rdx
            },
            loc: GameLocation::Id { id: AddrId::All { se: 51647, ae: 52520 }, offset: 0x14e },
            sig: signature![
                0x8b, 0xd0,
                0x48, 0x8d, 0x8f, ?, 0x00, 0x00, 0x00,
                0xff, 0x53, 0x18; 12
            ]
        },

        // As above, except this is for the function where the jump key is remapped.
        Descriptor::Patch {
            name: "CheckConditionForLegendarySkillAlt",
            enabled: settings::is_legendary_enabled,
            hook: Hook::Call12 {
                entry: check_condition_for_legendary_skill_alt_wrapper as *const u8,
                clobber: Register::Rdx
            },
            loc: GameLocation::Id { id: AddrId::All { se: 51638, ae: 52510 }, offset: 0x4d5 },
            sig: signature![
                0x8b, 0xd0,
                0x48, 0x8d, 0x8f, ?, 0x00, 0x00, 0x00,
                0xff, 0x53, 0x18; 12
            ]
        },

        // As above, except this is for the UI button display.
        Descriptor::Patch {
            name: "HideLegendaryButton",
            enabled: settings::is_legendary_enabled,
            hook: Hook::Call12 {
                entry: hide_legendary_button_wrapper as *const u8,
                clobber: Register::Rax
            },
            loc: GameLocation::Id { id: AddrId::All { se: 51654, ae: 52527 }, offset: 0x153 },
            sig: signature![
                0x48, 0x8b, 0x0d, ?, ?, ?, ?,
                0x48, 0x81, 0xc1, ?, 0x00, 0x00, 0x00,
                0x48, 0x8b, 0x01,
                0x41, 0x8b, 0xd7,
                0xff, 0x50, 0x18; 23
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
            enabled: settings::is_legendary_enabled,
            hook: Hook::Call12 {
                entry: clear_legendary_button_wrapper as *const u8,
                clobber: Register::Rax
            },
            loc: GameLocation::Id { id: AddrId::All { se: 51654, ae: 52527 }, offset: 0x16dd },
            sig: signature![
                0x48, 0x8b, 0x0d, ?, ?, ?, ?,
                0x48, 0x81, 0xc1, ?, 0x00, 0x00, 0x00,
                0x48, 0x8b, 0x01,
                0x41, 0x8b, 0xd7,
                0xff, 0x50, 0x08; 23
            ]
        }
    ];
}

/// Determines the real skill cap of the given skill.
#[no_mangle]
extern "system" fn get_skill_cap_hook(
    skill: c_int
) -> f32 {
    assert!(settings::is_skill_cap_enabled());
    settings::get_skill_cap(ActorAttribute::from_raw_skill(skill).unwrap())
}

/// Begins a calculation for weapon charge by setting the enchant cap to use the charge value.
#[no_mangle]
extern "system" fn max_charge_begin_hook(
    enchant_type: u32
) {
    const WEAPON_ENCHANT_TYPE: u32 = 0x29; // Defined by the game.
    if enchant_type == WEAPON_ENCHANT_TYPE {
        settings::use_enchant_charge_cap();
    }
}

/// Ends a calculation for weapon charge by returning the cap mode to magnitude, if necessary.
#[no_mangle]
extern "system" fn max_charge_end_hook() {
    settings::use_enchant_magnitude_cap();
}

///
/// Reimplements the enchantment charge point equation.
///
/// The original equation would fall apart for levels above 199, so this
/// implementation caps the level in the calculation to 199.
///
#[no_mangle]
extern "system" fn calculate_charge_points_per_use_hook(
    base_points: f32,
    max_charge: f32
) -> f32 {
    assert!(settings::is_enchant_patch_enabled());

    let cost_exponent = *ENCHANTING_COST_EXPONENT.get();
    let cost_base = *ENCHANTING_SKILL_COST_BASE.get();
    let cost_scale = *ENCHANTING_SKILL_COST_SCALE.get();
    let cost_mult = *ENCHANTING_SKILL_COST_MULT.get();
    let cap = settings::get_enchant_charge_cap();
    let enchanting_level = cap.min(player_avo_get_current(ActorAttribute::Enchanting));

    let base = cost_mult * base_points.powf(cost_exponent);
    if settings::is_enchant_charge_linear() {
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

/// Caps the formula results for each skill.
extern "system" fn player_avo_get_current_hook(
    av: *mut ActorValueOwner,
    attr: c_int
) -> f32 {
    assert!(settings::is_skill_formula_cap_enabled());

    let mut val = unsafe {
        // SAFETY: We are passing through the original arguments.
        player_avo_get_current_unchecked(av, attr)
    };

    if let Ok(skill) = ActorAttribute::from_raw_skill(attr) {
        val = val.min(settings::get_skill_formula_cap(skill)).max(0.0);
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
    natural_exp: bool,
    unk4: bool
) {
    assert!(settings::is_skill_exp_enabled());

    if let Ok(skill) = ActorAttribute::from_raw_skill(attr) {
        exp *= settings::get_skill_exp_mult(
            skill,
            player_avo_get_base(skill) as u32,
            get_player_level()
        );
    }

    unsafe {
        // SAFETY: We give it the same args, except the modified exp.
        improve_player_skill_points(skill_data, attr as c_int, exp, unk1, unk2, natural_exp, unk4);
    }
}

/// Adjusts the number of perks the player recieves at level-up.
#[no_mangle]
extern "system" fn modify_perk_pool_hook(
    count: i8
) {
    assert!(settings::is_perk_points_enabled());

    let pool = get_player_perk_pool();
    let delta = std::cmp::min(0xFF, settings::get_perk_delta(get_player_level()));
    let res = (pool.get() as i16) + (if count > 0 { delta as i16 } else { count as i16 });
    pool.set(std::cmp::max(0, std::cmp::min(0xff, res)) as u8);
}

/// Multiplies the exp gain of a level-up by the configured multiplier.
#[no_mangle]
extern "system" fn improve_level_exp_by_skill_level_hook(
    mut exp: f32,
    attr: c_int
) -> f32 {
    assert!(settings::is_level_exp_enabled());

    if let Ok(skill) = ActorAttribute::from_raw_skill(attr) {
        exp *= settings::get_level_exp_mult(
            skill,
            player_avo_get_base(skill) as u32,
            get_player_level()
        );
    }

    exp
}

///
/// Adjusts the attribute gain at each level-up based on the configured settings.
///
#[no_mangle]
extern "system" fn improve_attribute_when_level_up_hook(
    choice: c_int
) {
    assert!(settings::is_attr_points_enabled());
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
) -> f32 {
    assert!(settings::is_legendary_enabled());
    assert!(base_level >= 0.0);
    let base_val = *LEGENDARY_SKILL_RESET_VALUE.get();
    settings::get_post_legendary_skill_level(base_val, base_level)
}

///
/// Overwrites the check which determines when a skill can be legendary'd.
///
/// Due to how this function is injected, we return a "bool" based on the legendary
/// threshold. If the condition is valid, we return the threshold. Otherwise, we
/// return threshold - 1.
///
#[no_mangle]
extern "system" fn check_condition_for_legendary_skill_hook(
    skill: c_int
) -> f32 {
    assert!(settings::is_legendary_enabled());
    let skill = ActorAttribute::from_raw_skill(skill).unwrap();

    if settings::is_legendary_available(player_avo_get_base(skill) as u32) {
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
#[no_mangle]
extern "system" fn hide_legendary_button_hook(
    skill: c_int
) -> f32 {
    assert!(settings::is_legendary_enabled());
    let skill = ActorAttribute::from_raw_skill(skill).unwrap();

    if settings::is_legendary_button_visible(player_avo_get_base(skill) as u32) {
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
#[no_mangle]
extern "system" fn clear_legendary_button_hook(
    skill: c_int
) -> f32 {
    assert!(settings::is_legendary_enabled());

    if let Ok(skill) = ActorAttribute::from_raw_skill(skill) {
        let level = player_avo_get_base(skill);
        let game_vis = level >= BASE_LEGENDARY_THRESHOLD;
        let mod_vis = settings::is_legendary_button_visible(level as u32);

        if game_vis == mod_vis {
            level
        } else if game_vis { // visible, but shouldn't be.
            BASE_LEGENDARY_THRESHOLD - 1.0
        } else { // invisible, but shouldn't be.
            BASE_LEGENDARY_THRESHOLD
        }
    } else {
        // Some other perk menu. E.g. vampire or werewolf
        unsafe { player_avo_get_base_unchecked(get_player_avo(), skill) }
    }
}
