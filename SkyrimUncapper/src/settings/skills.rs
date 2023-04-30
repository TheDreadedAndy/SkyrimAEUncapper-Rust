//!
//! @file skills.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Provides traits and structures for reading skill-based configuration from INI files.
//! @bug No known bugs.
//!

use plugin_ini::Ini;

use super::config::IniNamedReadable;
use crate::skyrim::{ActorAttribute, SkillIterator, SKILL_COUNT};

pub trait IniSkillReadable {
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

/// Manages per-skill setting groups, allowing them to be read in together.
#[derive(Default)]
pub struct IniSkillManager<T: Default>([T; SKILL_COUNT]);

impl<T: IniSkillReadable + Default> IniSkillManager<T> {
    /// Gets the configuration item for the given skill.
    pub fn get(
        &self,
        skill: ActorAttribute
    ) -> &T {
        &self.0[skill.skill_slot()]
    }
}

impl<T: IniSkillReadable + Default> IniNamedReadable for IniSkillManager<T> {
    type Value = <T as IniSkillReadable>::Value;

    fn read_ini_named(
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
