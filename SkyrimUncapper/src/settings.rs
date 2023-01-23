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

use once_cell::sync::OnceCell;
use ini::Ini;
use skse64::errors::skse_assert;

use field::IniField;
use skills::IniSkillManager;
use leveled::LeveledIniSection;
use config::{DefaultIniSection, DefaultIniField, IniDefaultReadable};

struct GeneralSettings {
    skill_caps_en: DefaultIniField<IniField<bool>>,
    skill_formula_caps_en: DefaultIniField<IniField<bool>>,
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
    skill_exp_gain_mults: DefaultIniSection<IniSkillManager<IniField<f32>>>,
    skill_exp_gain_mults_with_skills: DefaultIniSection<IniSkillManager<LeveledIniSection<f32>>>,
    skill_exp_gain_mults_with_pc_lvl: DefaultIniSection<IniSkillManager<LeveledIniSection<f32>>>,
    level_exp_gain_mults: DefaultIniSection<IniSkillManager<IniField<f32>>>,
    level_exp_gain_mults_with_skills: DefaultIniSection<IniSkillManager<LeveledIniSection<f32>>>,
    level_exp_gain_mults_with_pc_lvl: DefaultIniSection<IniSkillManager<LeveledIniSection<f32>>>,
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

/// Holds the global settings configuration, which is created when init() is called.
static SETTINGS: OnceCell<Settings> = OnceCell::new();

impl Settings {
    /// Creates a new settings structure, with default values for missing fields.
    fn new() -> Self {
        const GEN_SEC: &str = "";
        const EN_SEC: &str = "";
        const LEG_SEC: &str = "";

        Self {
            general: GeneralSettings {
                skill_caps_en: DefaultIniField::new(GEN_SEC, "bUseSkillCaps", true),
                skill_formula_caps_en: DefaultIniField::new(GEN_SEC, "bUseSkillFormulaCaps", true),
                enchanting_patch_en: DefaultIniField::new(GEN_SEC, "bUseEnchanterCaps", true),
                skill_exp_mults_en: DefaultIniField::new(GEN_SEC, "bUseSkillExpGainMults", true),
                level_exp_mults_en: DefaultIniField::new(GEN_SEC, "bUsePcLevelSkillExpMults", true),
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
                hide_button: DefaultIniField::new(LEG_SEC, "bHideLegendaryButton", true),
                skill_level_en: DefaultIniField::new(LEG_SEC, "iSkillLevelEnableLegendary", 100),
                skill_level_after: DefaultIniField::new(LEG_SEC, "iSkillLevelAfterLegendary", 0),
            },
            skill_caps: DefaultIniSection::new("SkillCaps", 100),
            skill_formula_caps: DefaultIniSection::new("SkillFormulaCaps", 100),
            skill_exp_gain_mults: DefaultIniSection::new("SkillExpGainMults", 1.00),
            skill_exp_gain_mults_with_skills: DefaultIniSection::new(
                "SkillExpGainMults\\BaseSkillLevel",
                1.00
            ),
            skill_exp_gain_mults_with_pc_lvl: DefaultIniSection::new(
                "SkillExpGainMults\\CharacterLevel",
                1.00
            ),
            level_exp_gain_mults: DefaultIniSection::new("LevelSkillExpMults", 1.00),
            level_exp_gain_mults_with_skills: DefaultIniSection::new(
                "LevelSkillExpMults\\BaseSkillLevel",
                1.00
            ),
            level_exp_gain_mults_with_pc_lvl: DefaultIniSection::new(
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
            cw_at_sp_lvl_up: DefaultIniSection::new("CarryWeightAtStaminaLevelUp", 0)
        }
    }

    /// Reads in the settings from the given INI file.
    fn read_ini(
        &mut self,
        ini: &Ini
    ) {
        self.general.skill_caps_en.read_ini_default(ini);
        self.general.skill_formula_caps_en.read_ini_default(ini);
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
        self.skill_exp_gain_mults.read_ini_default(ini);
        self.skill_exp_gain_mults_with_skills.read_ini_default(ini);
        self.skill_exp_gain_mults_with_pc_lvl.read_ini_default(ini);
        self.level_exp_gain_mults.read_ini_default(ini);
        self.level_exp_gain_mults_with_skills.read_ini_default(ini);
        self.level_exp_gain_mults_with_pc_lvl.read_ini_default(ini);
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
) -> Result<(), ()> {
    let ini = Ini::load_from_file(path).map_err(|_| ())?;
    let mut settings = Settings::new();
    settings.read_ini(&ini);
    skse_assert!(SETTINGS.set(settings).is_ok());
    Ok(())
}

/// Checks if the skill cap patches are enabled.
pub fn is_skill_cap_enabled() -> bool {
    SETTINGS.get().unwrap().general.skill_caps_en.get()
}

/// Checks if the skill formula cap patches are enabled.
pub fn is_skill_formula_cap_enabled() -> bool {
    SETTINGS.get().unwrap().general.skill_formula_caps_en.get()
}

/// Checks if the enchanting patches are enabled.
pub fn is_enchant_patch_enabled() -> bool {
    SETTINGS.get().unwrap().general.enchanting_patch_en.get()
}

/// Checks if the skill exp patches are enabled.
pub fn is_skill_exp_enabled() -> bool {
    SETTINGS.get().unwrap().general.skill_exp_mults_en.get()
}

/// Checks if the level exp patches are enabled.
pub fn is_level_exp_enabled() -> bool {
    SETTINGS.get().unwrap().general.level_exp_mults_en.get()
}

/// Checks if the perk point patches are enabled.
pub fn is_perk_points_enabled() -> bool {
    SETTINGS.get().unwrap().general.perk_points_en.get()
}

/// Checks if the attribute point patches are enabled.
pub fn is_attr_points_enabled() -> bool {
    SETTINGS.get().unwrap().general.attr_points_en.get()
}

/// Checks if the legendary skill patches are enabled.
pub fn is_legendary_enabled() -> bool {
    SETTINGS.get().unwrap().general.legendary_en.get()
}
