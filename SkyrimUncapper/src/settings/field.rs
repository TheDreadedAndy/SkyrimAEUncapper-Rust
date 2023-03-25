//!
//! @file field.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Wraps a single field in an INI file.
//! @bug No known bugs.
//!

use std::fmt::Debug;
use std::str::FromStr;

use plugin_ini::Ini;
use skse64::log::skse_message;

use crate::skyrim::{ActorAttribute, HungarianAttribute};
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

impl<T: Copy + FromStr + Default> IniUnnamedReadable for IniField<T>
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
        let val = ini.get(section, name).unwrap_or_else(|| {
            skse_message!("[WARNING] Failed to load INI value {}: {}", section, name);
            default
        });

        self.0 = Some(val);
    }
}

impl<T: Copy + FromStr + Default + HungarianAttribute> IniSkillReadable for IniField<T>
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
        self.read_ini_unnamed(ini, section, T::hungarian_attr(skill), default);
    }
}
