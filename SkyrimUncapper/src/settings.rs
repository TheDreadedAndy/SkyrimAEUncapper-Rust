//!
//! @file settings.rs
//! @author Andrew Spaulding (Kasplat)
//! @author Kassent
//! @brief Loads and operates on the settings specified in the INI file.
//! @bug No known bugs.
//!

use core::fmt::Debug;
use core::str::FromStr;
use core::ffi::CStr;
use alloc::vec::Vec;
use alloc::string::String;

use core_util::Later;
use libskyrim::ini::Ini;
use libskyrim::log::{skse_message, skse_warning};

use crate::skyrim::{ActorAttribute, SkillIterator, HungarianAttribute, SKILL_COUNT};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Ini reading traits
////////////////////////////////////////////////////////////////////////////////////////////////////
//
// These traits describe how to read a specific field type in an INI in a way which minimizes the
// amount of code which must be duplicated whenever a new field is added to the INI.

trait IniReadableField {
    /// @brief The type of the underlying values.
    type Value: Copy;

    ///
    /// @brief Reads the value of the config item from the given section and key of the INI.
    /// @param ini The INI to read from.
    /// @param section The section of the INI to read from.
    /// @param name The key in the field to read from.
    /// @param default The default vaule to assume if none is available.
    ///
    fn read_ini_field(&mut self, ini: &Ini, section: &str, name: &str, default: Self::Value);
}

trait IniReadableSection {
    /// @brief The type of the underlying values.
    type Value: Copy;

    ///
    /// @brief Reads the value of the config item from the given section of the given INI.
    /// @param ini The INI to read from.
    /// @param section The section of the INI to read from.
    /// @param default The default value to assume if none is available.
    ///
    fn read_ini_section(&mut self, ini: &Ini, section: &str, default: Self::Value);
}

trait IniReadableSkill {
    type Value: Copy;

    ///
    /// @brief Reads a value from the given INI and section for the given skill.
    /// @param ini The INI to read from.
    /// @param section The section of the INI to read from.
    /// @param skill The skill which should be read.
    /// @param default The default value to assume if none is available.
    ///
    fn read_ini_skill(
        &mut self,
        ini: &Ini,
        section: &str,
        skill: ActorAttribute,
        default: Self::Value
    );
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Ini field type definitions and implementations
////////////////////////////////////////////////////////////////////////////////////////////////////
//
// These types implement the INI traits and define ways to add specific types of configuration
// fields and sections to the INI file.

/// Wraps a field which can be loaded from an INI file.
#[derive(Default)]
pub struct IniField<T: Default>(Option<T>);

/// Holds a level and setting pair in the list.
struct LevelItem<T> {
    level : u32,
    item  : T
}

/// Holds a setting which is configured on a per-level basis.
#[derive(Default)]
pub struct LeveledIniSection<T>(Vec<LevelItem<T>>);

/// Manages per-skill setting groups, allowing them to be read in together.
#[derive(Default)]
pub struct IniSkillManager<T: Default>([T; SKILL_COUNT]);

////////////////////////////////////////////////////////////////////////////////////////////////////

impl<T: Copy + Default> IniField<T> {
    /// Gets the configured value for this field.
    pub fn get(
        &self
    ) -> T {
        self.0.unwrap()
    }
}

impl<T: Copy + FromStr + Default> IniReadableField for IniField<T>
    where <T as FromStr>::Err: Debug
{
    type Value = T;
    fn read_ini_field(
        &mut self,
        ini: &Ini,
        section: &str,
        name: &str,
        default: Self::Value
    ) {
        let val = ini.get(section, name).unwrap_or_else(|| {
            skse_message!("[WARNING] Failed to load INI value {}: {}", section, name);
            default
        });

        self.0 = Some(val);
    }
}

impl<T: Copy + FromStr + Default + HungarianAttribute> IniReadableSkill for IniField<T>
    where <T as FromStr>::Err: core::fmt::Debug
{
    type Value = T;
    fn read_ini_skill(
        &mut self,
        ini: &Ini,
        section: &str,
        skill: ActorAttribute,
        default: Self::Value
    ) {
        self.read_ini_field(ini, section, T::hungarian_attr(skill), default);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

impl<T: Copy> LeveledIniSection<T> {
    ///
    /// Finds the value closest to the given level in the list.
    ///
    /// Note that only values whose level is less than or equal to the given level
    /// will be considered.
    ///
    pub fn get_nearest(
        &self,
        level: u32
    ) -> T {
        assert!(self.0.len() > 0);

        let (mut lo, mut hi): (usize, usize) = (0, self.0.len());
        let mut mid = lo + ((hi - lo) >> 1);
        while lo < hi {
            assert!(mid < self.0.len());
            if (self.0[mid].level <= level)
                    && ((mid + 1 == self.0.len()) || (level < self.0[mid + 1].level)) {
                return self.0[mid].item;
            } else if level < self.0[mid].level {
                hi = mid;
            } else {
                assert!((level > self.0[mid].level) || (level >= self.0[mid + 1].level));
                lo = mid + 1;
            }

            mid = lo + ((hi - lo) >> 1);
        }

        // If no direct match was found, return the closest lo item.
        return self.0[lo].item;
    }

    ///
    /// Adds an item to the leveled setting list.
    ///
    /// If the given item is already in the setting list, it will not be added again.
    ///
    fn add(
        &mut self,
        level: u32,
        item: T
    ) {
        // Store the items in sorted order, so we can binary search for the nearest later.
        let (mut lo, mut hi): (usize, usize) = (0, self.0.len());
        let mut mid: usize = lo + ((hi - lo) >> 1);
        while lo < hi {
            assert!(mid < self.0.len());
            if level < self.0[mid].level {
                hi = mid;
            } else if level > self.0[mid].level {
                lo = mid + 1;
            } else {
                return;
            }

            mid = lo + ((hi - lo) >> 1);
        }

        // Insert before the final hi element.
        assert!(hi <= self.0.len());
        self.0.insert(hi, LevelItem { level, item });
    }
}

impl LeveledIniSection<f32> {
    ///
    /// Accumulates the values across all previous levels, and determines
    /// what the increment from the last level was.
    ///
    /// This function is intended to be used for the calculation of partial
    /// perk point awards.
    ///
    pub fn get_cumulative_delta(
        &self,
        level: u32
    ) -> u32 {
        assert!(self.0.len() > 0);

        let mut acc: f32 = 0.0;
        let mut pacc: f32 = 0.0;
        let mut i = 0;
        while (i < self.0.len()) && (self.0[i].level <= level) {
            // Update the accumulation. Note the exclusize upper bound on level.
            let bound = if (i + 1) < self.0.len() { self.0[i + 1].level } else { level + 1 };
            let this_level = core::cmp::min(level + 1, bound);
            acc += ((this_level - self.0[i].level) as f32) * self.0[i].item;
            pacc = acc - self.0[i].item;
            i += 1;
        }

        return (acc as u32) - (pacc as u32);
    }
}

impl<T: Copy + FromStr> IniReadableSection for LeveledIniSection<T>
    where <T as FromStr>::Err: core::fmt::Debug
{
    type Value = T;
    fn read_ini_section(
        &mut self,
        ini: &Ini,
        section: &str,
        default: Self::Value
    ) {
        if let Ok(sec) = ini.section(section) {
            for field in sec.fields() {
                let level = if let Ok(l) = u32::from_str(field.name()) {
                    l
                } else {
                    skse_message!("[WARNING] Unable to convert {} to a u32; skipped", field.name());
                    continue;
                };

                let item = if let Some(i) = field.value() {
                    i
                } else {
                    skse_message!(
                        "[WARNING] Unabled to convert {} to value type; skipped",
                        field.value::<String>().as_ref().map(|s| s.as_ref()).unwrap_or("None")
                    );
                    continue;
                };

                self.add(level, item);
            }

            if self.0.len() == 0 {
                skse_message!("[WARNING] No values for in INI file for section {}", section);
                self.add(0, default);
            }
        } else {
            skse_message!("[WARNING] Unable to find section [{}] in INI file", section);
            self.add(0, default);
        }

        self.0.shrink_to_fit();
    }
}

impl<T: Copy + FromStr> IniReadableSkill for LeveledIniSection<T>
    where <T as FromStr>::Err: core::fmt::Debug
{
    type Value = T;
    fn read_ini_skill(
        &mut self,
        ini: &Ini,
        section: &str,
        skill: ActorAttribute,
        default: Self::Value
    ) {
        let section = String::from_str(section).unwrap() + "\\" + skill.name();
        self.read_ini_section(ini, &section, default);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

impl<T: Default> IniSkillManager<T> {
    /// Gets the configuration item for the given skill.
    pub fn get(
        &self,
        skill: ActorAttribute
    ) -> &T {
        &self.0[skill.skill_slot()]
    }
}

impl<T: IniReadableSkill + Default> IniReadableSection for IniSkillManager<T> {
    type Value = <T as IniReadableSkill>::Value;

    fn read_ini_section(
        &mut self,
        ini: &Ini,
        section: &str,
        default: Self::Value
    ) {
        for skill in SkillIterator::new() {
            self.0[skill.skill_slot()].read_ini_skill(ini, section, skill, default);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

const DEFAULT_INI_LZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/SkyrimUncapper.ini.lz"));

/// Manages the loading of a skill multiplier, which contains both a base and offset multiplier.
#[derive(Default, Copy, Clone)]
pub struct SkillMult {
    pub base: f32,
    pub offset: f32
}

#[derive(Default)]
pub struct GeneralSettings {
    pub skill_caps_en           : IniField<bool>,
    pub skill_formula_caps_en   : IniField<bool>,
    pub skill_formula_ui_fix_en : IniField<bool>,
    pub enchanting_patch_en     : IniField<bool>,
    pub skill_exp_mults_en      : IniField<bool>,
    pub level_exp_mults_en      : IniField<bool>,
    pub perk_points_en          : IniField<bool>,
    pub attr_points_en          : IniField<bool>,
    pub legendary_en            : IniField<bool>
}

#[derive(Default)]
pub struct EnchantSettings {
    pub magnitude_cap     : IniField<u32>,
    pub charge_cap        : IniField<u32>,
    pub use_linear_charge : IniField<bool>
}

#[derive(Default)]
pub struct LegendarySettings {
    pub keep_skill_level  : IniField<bool>,
    pub hide_button       : IniField<bool>,
    pub skill_level_en    : IniField<u32>,
    pub skill_level_after : IniField<u32>
}

/// Contains all the configuration settings loaded in from the INI file.
#[derive(Default)]
pub struct Settings {
    pub general                     : GeneralSettings,
    pub enchant                     : EnchantSettings,
    pub legendary                   : LegendarySettings,
    pub skill_caps                  : IniSkillManager<IniField<u32>>,
    pub skill_formula_caps          : IniSkillManager<IniField<u32>>,
    pub skill_exp_mults             : IniSkillManager<IniField<SkillMult>>,
    pub skill_exp_mults_with_skills : IniSkillManager<LeveledIniSection<SkillMult>>,
    pub skill_exp_mults_with_pc_lvl : IniSkillManager<LeveledIniSection<SkillMult>>,
    pub level_exp_mults             : IniSkillManager<IniField<f32>>,
    pub level_exp_mults_with_skills : IniSkillManager<LeveledIniSection<f32>>,
    pub level_exp_mults_with_pc_lvl : IniSkillManager<LeveledIniSection<f32>>,
    pub perks_at_lvl_up             : LeveledIniSection<f32>,
    pub hp_at_lvl_up                : LeveledIniSection<u32>,
    pub hp_at_mp_lvl_up             : LeveledIniSection<u32>,
    pub hp_at_sp_lvl_up             : LeveledIniSection<u32>,
    pub mp_at_lvl_up                : LeveledIniSection<u32>,
    pub mp_at_hp_lvl_up             : LeveledIniSection<u32>,
    pub mp_at_sp_lvl_up             : LeveledIniSection<u32>,
    pub sp_at_lvl_up                : LeveledIniSection<u32>,
    pub sp_at_hp_lvl_up             : LeveledIniSection<u32>,
    pub sp_at_mp_lvl_up             : LeveledIniSection<u32>,
    pub cw_at_hp_lvl_up             : LeveledIniSection<u32>,
    pub cw_at_mp_lvl_up             : LeveledIniSection<u32>,
    pub cw_at_sp_lvl_up             : LeveledIniSection<u32>
}

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

/// Holds the global settings configuration, which is created when init() is called.
pub static SETTINGS: Later<Settings> = Later::new();

impl Settings {
    /// Creates a new settings structure from the given INI.
    ///
    /// If a field/section is missing in the given INI, a default value is used.
    fn new(
        ini: &Ini
    ) -> Self {
        const GEN_SEC : &'static str = "General";
        const EN_SEC  : &'static str = "Enchanting";
        const LEG_SEC : &'static str = "LegendarySkill";

        const DEFAULT_SKILL_EXP_MULT : SkillMult = SkillMult { base: 1.0, offset: 1.0 };
        const DEFAULT_LEVEL_EXP_MULT : f32       = 1.0;

        let mut ret = Self::default();

        ////////////////////////////////////////////////////////////////////////////////////////////
        // General settings
        ////////////////////////////////////////////////////////////////////////////////////////////

        ret.general.skill_caps_en.read_ini_field(ini, GEN_SEC, "bUseSkillCaps", true);
        ret.general.skill_formula_caps_en.read_ini_field(
            ini,
            GEN_SEC,
            "bUseSkillFormulaCaps",
            true
        );
        ret.general.skill_formula_ui_fix_en.read_ini_field(
            ini,
            GEN_SEC,
            "bUseSkillFormulaCapsUIFix",
            true
        );
        ret.general.enchanting_patch_en.read_ini_field(ini, GEN_SEC, "bUseEnchanterCaps", true);
        ret.general.skill_exp_mults_en.read_ini_field(ini, GEN_SEC, "bUseSkillExpGainMults", true);
        ret.general.level_exp_mults_en.read_ini_field(
            ini,
            GEN_SEC,
            "bUsePCLevelSkillExpMults",
            true
        );
        ret.general.perk_points_en.read_ini_field(ini, GEN_SEC, "bUsePerksAtLevelUp", true);
        ret.general.attr_points_en.read_ini_field(ini, GEN_SEC, "bUseAttributesAtLevelUp", true);
        ret.general.legendary_en.read_ini_field(ini, GEN_SEC, "bUseLegendarySettings", true);

        ////////////////////////////////////////////////////////////////////////////////////////////
        // Enchanting settings
        ////////////////////////////////////////////////////////////////////////////////////////////

        ret.enchant.magnitude_cap.read_ini_field(ini, EN_SEC, "iMagnitudeLevelCap", 100);
        ret.enchant.charge_cap.read_ini_field(ini, EN_SEC, "iChargeLevelCap", 199);
        ret.enchant.use_linear_charge.read_ini_field(ini, EN_SEC, "bUseLinearChargeFormula", false);

        ////////////////////////////////////////////////////////////////////////////////////////////
        // Legendary settings
        ////////////////////////////////////////////////////////////////////////////////////////////

        ret.legendary.keep_skill_level.read_ini_field(
            ini,
            LEG_SEC,
            "bLegendaryKeepSkillLevel",
            false
        );
        ret.legendary.hide_button.read_ini_field(ini, LEG_SEC, "bHideLegendaryButton", false);
        ret.legendary.skill_level_en.read_ini_field(
            ini,
            LEG_SEC,
            "iSkillLevelEnableLegendary",
            100
        );
        ret.legendary.skill_level_after.read_ini_field(
            ini,
            LEG_SEC,
            "iSkillLevelAfterLegendary",
            0
        );

        ////////////////////////////////////////////////////////////////////////////////////////////
        // Grouped sections
        ////////////////////////////////////////////////////////////////////////////////////////////

        ret.skill_caps.read_ini_section(ini, "SkillCaps", 100);
        ret.skill_formula_caps.read_ini_section(ini, "SkillFormulaCaps", 100);
        ret.skill_exp_mults.read_ini_section(ini, "SkillExpGainMults", DEFAULT_SKILL_EXP_MULT);
        ret.skill_exp_mults_with_skills.read_ini_section(
            ini,
            "SkillExpGainMults\\BaseSkillLevel",
            DEFAULT_SKILL_EXP_MULT
        );
        ret.skill_exp_mults_with_pc_lvl.read_ini_section(
            ini,
            "SkillExpGainMults\\CharacterLevel",
            DEFAULT_SKILL_EXP_MULT
        );
        ret.level_exp_mults.read_ini_section(ini, "LevelSkillExpMults", DEFAULT_LEVEL_EXP_MULT);
        ret.level_exp_mults_with_skills.read_ini_section(
            ini,
            "LevelSkillExpMults\\BaseSkillLevel",
            DEFAULT_LEVEL_EXP_MULT
        );
        ret.level_exp_mults_with_pc_lvl.read_ini_section(
            ini,
            "LevelSkillExpMults\\CharacterLevel",
            DEFAULT_LEVEL_EXP_MULT
        );
        ret.perks_at_lvl_up.read_ini_section(ini, "PerksAtLevelUp", 1.00);
        ret.hp_at_lvl_up.read_ini_section(ini, "HealthAtLevelUp", 10);
        ret.hp_at_mp_lvl_up.read_ini_section(ini, "HealthAtMagickaLevelUp", 0);
        ret.hp_at_sp_lvl_up.read_ini_section(ini, "HealthAtStaminaLevelUp", 0);
        ret.mp_at_lvl_up.read_ini_section(ini, "MagickaAtLevelUp", 10);
        ret.mp_at_hp_lvl_up.read_ini_section(ini, "MagickaAtHealthLevelUp", 0);
        ret.mp_at_sp_lvl_up.read_ini_section(ini, "MagickaAtStaminaLevelUp", 0);
        ret.sp_at_lvl_up.read_ini_section(ini, "StaminaAtLevelUp", 10);
        ret.sp_at_hp_lvl_up.read_ini_section(ini, "StaminaAtHealthLevelUp", 0);
        ret.sp_at_mp_lvl_up.read_ini_section(ini, "StaminaAtMagickaLevelUp", 0);
        ret.cw_at_hp_lvl_up.read_ini_section(ini, "CarryWeightAtHealthLevelUp", 0);
        ret.cw_at_mp_lvl_up.read_ini_section(ini, "CarryWeightAtMagickaLevelUp", 0);
        ret.cw_at_sp_lvl_up.read_ini_section(ini, "CarryWeightAtStaminaLevelUp", 5);

        return ret;
    }
}

/// Attempts to load the settings structure from the given INI file.
pub fn init(
    path: &CStr
) {
    skse_message!("Loading config file: {}", String::from_utf8_lossy(path.to_bytes()));

    let default_ini = Ini::from_str(unsafe {
        // SAFETY: We know this file was given as UTF8 text when it was compressed.
        &String::from_utf8_unchecked(deflate::decompress(DEFAULT_INI_LZ))
    }).unwrap();

    // Read the configuration from the file.
    let ini = Ini::from_path(path);
    if let Ok(mut ini) = ini {
        // Update the file with missing fields, if necessary.
        if ini.update(&default_ini) {
            // If missing fields were added, update the INI file.
            assert!(
                ini.write_file(path).is_ok(),
                "[ERROR] Failed to write to INI file. Please ensure Skyrim has \
                         permission to use the plugin directory."
            );

            skse_warning!("The INI file has been updated.");
        }

        SETTINGS.init(Settings::new(&ini));
    } else {
        skse_warning!("Could not load INI file. Defaults will be used.");
        SETTINGS.init(Settings::new(&default_ini));
    }

    skse_message!("Done initializing settings!");
}
