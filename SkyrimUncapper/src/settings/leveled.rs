//!
//! @file leveled.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Defines the structure used to manage leveled settings.
//! @bug No known bugs.
//!

use std::vec::Vec;
use std::str::FromStr;

use plugin_ini::Ini;
use skse64::log::skse_message;

use crate::skyrim::ActorAttribute;
use super::config::IniNamedReadable;
use super::skills::IniSkillReadable;

/// Holds a level and setting pair in the list.
struct LevelItem<T> {
    level: u32,
    item: T
}

/// Holds a setting which is configured on a per-level basis.
#[derive(Default)]
pub struct LeveledIniSection<T>(Vec<LevelItem<T>>);

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
            let this_level = std::cmp::min(level + 1, bound);
            acc += ((this_level - self.0[i].level) as f32) * self.0[i].item;
            pacc = acc - self.0[i].item;
            i += 1;
        }

        return (acc as u32) - (pacc as u32);
    }
}

impl<T: Copy + FromStr> IniNamedReadable for LeveledIniSection<T>
    where <T as FromStr>::Err: std::fmt::Debug
{
    type Value = T;
    fn read_ini_named(
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

impl<T: Copy + FromStr> IniSkillReadable for LeveledIniSection<T>
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
        let section = String::from_str(section).unwrap() + "\\" + skill.name();
        self.read_ini_named(ini, &section, default);
    }
}
