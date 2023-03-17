//!
//! @file settings.rs
//! @author Andrew Spaulding (Kasplat)
//! @author Kassent
//! @brief Loads and operates on the settings specified in the INI file.
//! @bug No known bugs.
//!

mod config;
mod skills;
mod field;
mod leveled;

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::str::FromStr;

use later::Later;
use plugin_ini::Ini;
use skse64::log::{skse_message, skse_warning};

use field::IniField;
use skills::IniSkillManager;
use leveled::LeveledIniSection;
use config::{DefaultIniSection, DefaultIniField, IniDefaultReadable};
use crate::skyrim::ActorAttribute;

const DEFAULT_INI_LZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SkyrimUncapper.ini.lz"));

/// Manages the loading of a skill multiplier, which contains both a base and offset multiplier.
#[derive(Default, Copy, Clone)]
pub (in crate) struct SkillMult {
    base: f32,
    offset: f32
}

struct GeneralSettings {
    skill_caps_en: DefaultIniField<IniField<bool>>,
    skill_formula_caps_en: DefaultIniField<IniField<bool>>,
    skill_formula_ui_fix_en: DefaultIniField<IniField<bool>>,
    enchanting_patch_en: DefaultIniField<IniField<bool>>,
    skill_exp_mults_en: DefaultIniField<IniField<bool>>,
    level_exp_mults_en: DefaultIniField<IniField<bool>>,
    perk_points_en: DefaultIniField<IniField<bool>>,
    attr_points_en: DefaultIniField<IniField<bool>>,
    legendary_en: DefaultIniField<IniField<bool>>
}

struct EnchantSettings {
    magnitude_cap: DefaultIniField<IniField<u32>>,
    charge_cap: DefaultIniField<IniField<u32>>,
    use_linear_charge: DefaultIniField<IniField<bool>>
}

struct LegendarySettings {
    keep_skill_level: DefaultIniField<IniField<bool>>,
    hide_button: DefaultIniField<IniField<bool>>,
    skill_level_en: DefaultIniField<IniField<u32>>,
    skill_level_after: DefaultIniField<IniField<u32>>
}

/// Contains all the configuration settings loaded in from the INI file.
struct Settings {
    general: GeneralSettings,
    enchant: EnchantSettings,
    legendary: LegendarySettings,
    skill_caps: DefaultIniSection<IniSkillManager<IniField<u32>>>,
    skill_formula_caps: DefaultIniSection<IniSkillManager<IniField<u32>>>,
    skill_exp_mults: DefaultIniSection<IniSkillManager<IniField<SkillMult>>>,
    skill_exp_mults_with_skills: DefaultIniSection<IniSkillManager<LeveledIniSection<SkillMult>>>,
    skill_exp_mults_with_pc_lvl: DefaultIniSection<IniSkillManager<LeveledIniSection<SkillMult>>>,
    level_exp_mults: DefaultIniSection<IniSkillManager<IniField<f32>>>,
    level_exp_mults_with_skills: DefaultIniSection<IniSkillManager<LeveledIniSection<f32>>>,
    level_exp_mults_with_pc_lvl: DefaultIniSection<IniSkillManager<LeveledIniSection<f32>>>,
    perks_at_lvl_up: DefaultIniSection<LeveledIniSection<f32>>,
    hp_at_lvl_up: DefaultIniSection<LeveledIniSection<u32>>,
    hp_at_mp_lvl_up: DefaultIniSection<LeveledIniSection<u32>>,
    hp_at_sp_lvl_up: DefaultIniSection<LeveledIniSection<u32>>,
    mp_at_lvl_up: DefaultIniSection<LeveledIniSection<u32>>,
    mp_at_hp_lvl_up: DefaultIniSection<LeveledIniSection<u32>>,
    mp_at_sp_lvl_up: DefaultIniSection<LeveledIniSection<u32>>,
    sp_at_lvl_up: DefaultIniSection<LeveledIniSection<u32>>,
    sp_at_hp_lvl_up: DefaultIniSection<LeveledIniSection<u32>>,
    sp_at_mp_lvl_up: DefaultIniSection<LeveledIniSection<u32>>,
    cw_at_hp_lvl_up: DefaultIniSection<LeveledIniSection<u32>>,
    cw_at_mp_lvl_up: DefaultIniSection<LeveledIniSection<u32>>,
    cw_at_sp_lvl_up: DefaultIniSection<LeveledIniSection<u32>>
}

/// By default, skill exp multiplication is disabled.
const DEFAULT_SKILL_EXP_MULT: SkillMult = SkillMult { base: 1.0, offset: 1.0 };

/// Holds the global settings configuration, which is created when init() is called.
static SETTINGS: Later<Settings> = Later::new();

/// Used to ensure that the max_charge critical section is not entered twice.
static IS_USING_CHARGE_CAP: AtomicBool = AtomicBool::new(false);

/// Allows for the optional loading of an offset multiplier.
impl FromStr for SkillMult {
    type Err = <f32 as FromStr>::Err;

    fn from_str(
        s: &str
    ) -> Result<Self, Self::Err> {
        if let Some(s) = s.split_once('/') {
            // Since there is a mode symbol, the offset has been specified in some way.
            if s.1.trim().is_empty() {
                // We have been asked to duplicate the first number for both.
                Ok(Self { base: f32::from_str(s.0)?, offset: f32::from_str(s.0)? })
            } else {
                // Both values were given.
                Ok(Self { base: f32::from_str(s.0)?, offset: f32::from_str(s.1)? })
            }
        } else {
            // Compat mode: Assume offset mult is 1.0 and parse as single float.
            Ok(Self { base: f32::from_str(s)?, offset: 1.0 })
        }
    }
}

impl Settings {
    /// Creates a new settings structure, with default values for missing fields.
    fn new() -> Self {
        const GEN_SEC: &'static str = "General";
        const EN_SEC: &'static str = "Enchanting";
        const LEG_SEC: &'static str = "LegendarySkill";

        Self {
            general: GeneralSettings {
                skill_caps_en: DefaultIniField::new(GEN_SEC, "bUseSkillCaps", true),
                skill_formula_caps_en: DefaultIniField::new(GEN_SEC, "bUseSkillFormulaCaps", true),
                skill_formula_ui_fix_en: DefaultIniField::new(
                    GEN_SEC,
                    "bUseSkillFormulaCapsUIFix",
                    true
                ),
                enchanting_patch_en: DefaultIniField::new(GEN_SEC, "bUseEnchanterCaps", true),
                skill_exp_mults_en: DefaultIniField::new(GEN_SEC, "bUseSkillExpGainMults", true),
                level_exp_mults_en: DefaultIniField::new(GEN_SEC, "bUsePCLevelSkillExpMults", true),
                perk_points_en: DefaultIniField::new(GEN_SEC, "bUsePerksAtLevelUp", true),
                attr_points_en: DefaultIniField::new(GEN_SEC, "bUseAttributesAtLevelUp", true),
                legendary_en: DefaultIniField::new(GEN_SEC, "bUseLegendarySettings", true)
            },
            enchant: EnchantSettings {
                magnitude_cap: DefaultIniField::new(EN_SEC, "iMagnitudeLevelCap", 100),
                charge_cap: DefaultIniField::new(EN_SEC, "iChargeLevelCap", 199),
                use_linear_charge: DefaultIniField::new(EN_SEC, "bUseLinearChargeFormula", false),
            },
            legendary: LegendarySettings {
                keep_skill_level: DefaultIniField::new(LEG_SEC, "bLegendaryKeepSkillLevel", false),
                hide_button: DefaultIniField::new(LEG_SEC, "bHideLegendaryButton", false),
                skill_level_en: DefaultIniField::new(LEG_SEC, "iSkillLevelEnableLegendary", 100),
                skill_level_after: DefaultIniField::new(LEG_SEC, "iSkillLevelAfterLegendary", 0),
            },
            skill_caps: DefaultIniSection::new("SkillCaps", 100),
            skill_formula_caps: DefaultIniSection::new("SkillFormulaCaps", 100),
            skill_exp_mults: DefaultIniSection::new("SkillExpGainMults", DEFAULT_SKILL_EXP_MULT),
            skill_exp_mults_with_skills: DefaultIniSection::new(
                "SkillExpGainMults\\BaseSkillLevel",
                DEFAULT_SKILL_EXP_MULT
            ),
            skill_exp_mults_with_pc_lvl: DefaultIniSection::new(
                "SkillExpGainMults\\CharacterLevel",
                DEFAULT_SKILL_EXP_MULT
            ),
            level_exp_mults: DefaultIniSection::new("LevelSkillExpMults", 1.00),
            level_exp_mults_with_skills: DefaultIniSection::new(
                "LevelSkillExpMults\\BaseSkillLevel",
                1.00
            ),
            level_exp_mults_with_pc_lvl: DefaultIniSection::new(
                "LevelSkillExpMults\\CharacterLevel",
                1.00
            ),
            perks_at_lvl_up: DefaultIniSection::new("PerksAtLevelUp", 1.00),
            hp_at_lvl_up: DefaultIniSection::new("HealthAtLevelUp", 10),
            hp_at_mp_lvl_up: DefaultIniSection::new("HealthAtMagickaLevelUp", 0),
            hp_at_sp_lvl_up: DefaultIniSection::new("HealthAtStaminaLevelUp", 0),
            mp_at_lvl_up: DefaultIniSection::new("MagickaAtLevelUp", 10),
            mp_at_hp_lvl_up: DefaultIniSection::new("MagickaAtHealthLevelUp", 0),
            mp_at_sp_lvl_up: DefaultIniSection::new("MagickaAtStaminaLevelUp", 0),
            sp_at_lvl_up: DefaultIniSection::new("StaminaAtLevelUp", 10),
            sp_at_hp_lvl_up: DefaultIniSection::new("StaminaAtHealthLevelUp", 0),
            sp_at_mp_lvl_up: DefaultIniSection::new("StaminaAtMagickaLevelUp", 0),
            cw_at_hp_lvl_up: DefaultIniSection::new("CarryWeightAtHealthLevelUp", 0),
            cw_at_mp_lvl_up: DefaultIniSection::new("CarryWeightAtMagickaLevelUp", 0),
            cw_at_sp_lvl_up: DefaultIniSection::new("CarryWeightAtStaminaLevelUp", 5)
        }
    }

    /// Reads in the settings from the given INI file.
    fn read_ini(
        &mut self,
        ini: &Ini
    ) {
        self.general.skill_caps_en.read_ini_default(ini);
        self.general.skill_formula_caps_en.read_ini_default(ini);
        self.general.skill_formula_ui_fix_en.read_ini_default(ini);
        self.general.enchanting_patch_en.read_ini_default(ini);
        self.general.skill_exp_mults_en.read_ini_default(ini);
        self.general.level_exp_mults_en.read_ini_default(ini);
        self.general.perk_points_en.read_ini_default(ini);
        self.general.attr_points_en.read_ini_default(ini);
        self.general.legendary_en.read_ini_default(ini);
        self.enchant.magnitude_cap.read_ini_default(ini);
        self.enchant.charge_cap.read_ini_default(ini);
        self.enchant.use_linear_charge.read_ini_default(ini);
        self.legendary.keep_skill_level.read_ini_default(ini);
        self.legendary.hide_button.read_ini_default(ini);
        self.legendary.skill_level_en.read_ini_default(ini);
        self.legendary.skill_level_after.read_ini_default(ini);
        self.skill_caps.read_ini_default(ini);
        self.skill_formula_caps.read_ini_default(ini);
        self.skill_exp_mults.read_ini_default(ini);
        self.skill_exp_mults_with_skills.read_ini_default(ini);
        self.skill_exp_mults_with_pc_lvl.read_ini_default(ini);
        self.level_exp_mults.read_ini_default(ini);
        self.level_exp_mults_with_skills.read_ini_default(ini);
        self.level_exp_mults_with_pc_lvl.read_ini_default(ini);
        self.perks_at_lvl_up.read_ini_default(ini);
        self.hp_at_lvl_up.read_ini_default(ini);
        self.hp_at_mp_lvl_up.read_ini_default(ini);
        self.hp_at_sp_lvl_up.read_ini_default(ini);
        self.mp_at_lvl_up.read_ini_default(ini);
        self.mp_at_hp_lvl_up.read_ini_default(ini);
        self.mp_at_sp_lvl_up.read_ini_default(ini);
        self.sp_at_lvl_up.read_ini_default(ini);
        self.sp_at_hp_lvl_up.read_ini_default(ini);
        self.sp_at_mp_lvl_up.read_ini_default(ini);
        self.cw_at_hp_lvl_up.read_ini_default(ini);
        self.cw_at_mp_lvl_up.read_ini_default(ini);
        self.cw_at_sp_lvl_up.read_ini_default(ini);
    }
}

/// Attempts to load the settings structure from the given INI file.
pub fn init(
    path: &Path
) {
    skse_message!("Loading config file: {}", path.display());

    // Read the configuration from the file.
    let ini = Ini::from_path(path);
    if ini.is_err() {
        skse_warning!("Could not load INI file. Defaults will be used.");
    }

    // Update the file with missing fields, if necessary.
    let mut ini = ini.unwrap();
    let default_ini = Ini::from_str(unsafe {
        // SAFETY: We know this file was given as UTF8 text when it was compressed.
        &String::from_utf8_unchecked(deflate::decompress(DEFAULT_INI_LZ))
    }).unwrap();
    if let Some(_) = ini.update(&default_ini) {
        // If missing fields were added, update the INI file.
        assert!(
            ini.write_file(path).is_ok(),
            "[ERROR] Failed to write to INI file. Please ensure Skyrim has permission to use the \
             plugin directory."
        );

        skse_warning!("The INI file has been updated.");
    }

    let mut settings = Settings::new();
    settings.read_ini(&ini);
    SETTINGS.init(settings);

    skse_message!("Done initializing settings!");
}

/// Checks if the skill cap patches are enabled.
pub fn is_skill_cap_enabled() -> bool {
    SETTINGS.general.skill_caps_en.get()
}

/// Checks if the skill formula cap patches are enabled.
pub fn is_skill_formula_cap_enabled() -> bool {
    SETTINGS.general.skill_formula_caps_en.get()
}

/// Checks if the skill formula cap UI fixes are enabled.
pub fn is_skill_formula_cap_ui_fix_enabled() -> bool {
    SETTINGS.general.skill_formula_caps_en.get() && SETTINGS.general.skill_formula_ui_fix_en.get()
}

/// Checks if the enchanting patches are enabled.
pub fn is_enchant_patch_enabled() -> bool {
    SETTINGS.general.enchanting_patch_en.get()
}

/// Checks if the skill exp patches are enabled.
pub fn is_skill_exp_enabled() -> bool {
    SETTINGS.general.skill_exp_mults_en.get()
}

/// Checks if the level exp patches are enabled.
pub fn is_level_exp_enabled() -> bool {
    SETTINGS.general.level_exp_mults_en.get()
}

/// Checks if the perk point patches are enabled.
pub fn is_perk_points_enabled() -> bool {
    SETTINGS.general.perk_points_en.get()
}

/// Checks if the attribute point patches are enabled.
pub fn is_attr_points_enabled() -> bool {
    SETTINGS.general.attr_points_en.get()
}

/// Checks if the legendary skill patches are enabled.
pub fn is_legendary_enabled() -> bool {
    SETTINGS.general.legendary_en.get()
}

/// Gets the level cap for the given skill.
pub fn get_skill_cap(
    skill: ActorAttribute
) -> f32 {
    SETTINGS.skill_caps.get(skill).get() as f32
}

/// Gets the formula cap for the given skill.
pub fn get_skill_formula_cap(
    skill: ActorAttribute
) -> f32 {
    let mut cap = SETTINGS.skill_formula_caps.get(skill).get() as f32;

    // Enforce the additional enchanting caps.
    if skill == ActorAttribute::Enchanting {
        let specific_cap = if IS_USING_CHARGE_CAP.load(Ordering::Relaxed) {
            SETTINGS.enchant.charge_cap.get() as f32
        } else {
            SETTINGS.enchant.magnitude_cap.get() as f32
        };

        cap = cap.min(specific_cap);
    }

    return cap;
}

/// Enables the use of the charge cap for the skill formula cap. It must be disabled when invoked.
pub fn use_enchant_charge_cap() {
    assert!(!IS_USING_CHARGE_CAP.swap(true, Ordering::Relaxed));
}

/// Disables the use of the charge cap for the skill formula cap, if it was enabled.
pub fn use_enchant_magnitude_cap() {
    let _ = IS_USING_CHARGE_CAP.compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed);
}

/// Gets the formula cap for weapon-charge enchantments.
pub fn get_enchant_charge_cap() -> f32 {
    (SETTINGS.enchant.charge_cap.get() as f32).min(199.0).min(
        SETTINGS.skill_formula_caps.get(ActorAttribute::Enchanting).get() as f32
    )
}

/// Checks if the weapon charge equation should use a linear charge amount increase per level.
pub fn is_enchant_charge_linear() -> bool {
    SETTINGS.enchant.use_linear_charge.get()
}

/// Calculates the skill exp gain multiplier for the given skill, skill level, and player level.
pub fn get_skill_exp_mult(
    skill: ActorAttribute,
    skill_level: u32,
    player_level: u32
) -> (f32, f32) {
    let base_mult = SETTINGS.skill_exp_mults.get(skill).get();
    let skill_mult = SETTINGS.skill_exp_mults_with_skills.get(skill).get_nearest(skill_level);
    let pc_mult = SETTINGS.skill_exp_mults_with_pc_lvl.get(skill).get_nearest(player_level);

    (base_mult.base * skill_mult.base * pc_mult.base,
     base_mult.offset * skill_mult.offset * pc_mult.offset)
}

/// Calculates the level exp gain multiplier for the given skill, skill level, and player level.
pub fn get_level_exp_mult(
    skill: ActorAttribute,
    skill_level: u32,
    player_level: u32
) -> f32 {
    let base_mult = SETTINGS.level_exp_mults.get(skill).get();
    let skill_mult = SETTINGS.level_exp_mults_with_skills.get(skill).get_nearest(skill_level);
    let pc_mult = SETTINGS.level_exp_mults_with_pc_lvl.get(skill).get_nearest(player_level);
    return base_mult * skill_mult * pc_mult;
}

/// Gets the number of perk points the player should receive for reaching the given level.
pub fn get_perk_delta(
    player_level: u32
) -> u32 {
    SETTINGS.perks_at_lvl_up.get_cumulative_delta(player_level)
}

/// Gets the number of (hp, mp, sp, cw) points the player should get for the given level and
/// attribute selection.
pub fn get_attribute_level_up(
    player_level: u32,
    attr: ActorAttribute
) -> (f32, f32, f32, f32) {
    match attr {
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
    }
}

/// Checks if the legendary button should be displayed above a skill with the given level.
pub fn is_legendary_button_visible(
    skill_level: u32
) -> bool {
    (skill_level >= SETTINGS.legendary.skill_level_en.get())
        && !(SETTINGS.legendary.hide_button.get())
}

/// Checks if the given skill level is high enough to legendary.
pub fn is_legendary_available(
    skill_level: u32
) -> bool {
    skill_level >= SETTINGS.legendary.skill_level_en.get()
}

/// Gets the level a skill should be set to after being legendaried.
pub fn get_post_legendary_skill_level(
    default_reset: f32,
    base_level: f32
) -> f32 {
    // Check if legendarying should reset the level at all.
    if SETTINGS.legendary.keep_skill_level.get() {
        return base_level;
    }

    // 0 in the conf file means we should use the default value.
    let mut reset_level = SETTINGS.legendary.skill_level_after.get() as f32;
    if reset_level == 0.0 {
        reset_level = default_reset;
    }

    // Don't allow legendarying to raise the skill level.
    reset_level.min(base_level)
}
