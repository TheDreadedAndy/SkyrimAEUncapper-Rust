//!
//! @file field.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Wraps a single field in an INI file.
//! @bug No known bugs.
//!

use std::fmt::Debug;
use std::str::FromStr;
use std::string::ToString;

use ini::Ini;

use crate::skyrim::ActorAttribute;
use super::config::IniUnnamedReadable;
use super::skills::IniSkillReadable;

/// Wraps a field which can be loaded from an INI file.
#[derive(Default)]
pub struct IniField<T: Default>(Option<T>);

impl<T: Copy + Default> IniField<T> {
    /// Gets the configured value for this field.
    pub fn get(
        &self
    ) -> T {
        self.0.unwrap()
    }
}

impl<T: Copy + FromStr + ToString + Default> IniUnnamedReadable for IniField<T>
    where <T as FromStr>::Err: Debug
{
    type Value = T;
    fn read_ini_unnamed(
        &mut self,
        ini: &Ini,
        section: &str,
        name: &str,
        default: Self::Value
    ) {
        self.0 = Some(
            T::from_str(ini.get_from_or(Some(section), name, &default.to_string())).unwrap()
        );
    }
}

impl<T: Copy + FromStr + ToString + Default> IniSkillReadable for IniField<T>
    where <T as FromStr>::Err: std::fmt::Debug
{
    type Value = T;
    fn read_ini_skill(
        &mut self,
        ini: &Ini,
        section: &str,
        skill: ActorAttribute,
        default: Self::Value
    ) {
        self.read_ini_unnamed(ini, section, skill.name(), default);
    }
}
