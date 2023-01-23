//!
//! @file config.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Provides traits for reading in values from INI files.
//! @bug No known bugs.
//!

use std::ops::Deref;
use ini::Ini;

pub trait IniNamedReadable {
    /// @brief The type of the underlying values.
    type Value: Copy;

    ///
    /// @brief Reads the value of the config item from the given section of the given INI.
    /// @param ini The INI to read from.
    /// @param section The section of the INI to read from.
    /// @param default The default value to assume if none is available.
    ///
    fn read_ini_named(&mut self, ini: &Ini, section: &str, default: Self::Value);
}

pub trait IniUnnamedReadable {
    /// @brief The type of the underlying vaules.
    type Value: Copy;

    ///
    /// @brief Reads the value of the config item from the given section and key of the INI.
    /// @param ini The INI to read from.
    /// @param section The section of the INI to read from.
    /// @param name The key in the field to read from.
    /// @param default The default vaule to assume if none is available.
    ///
    fn read_ini_unnamed(&mut self, ini: &Ini, section: &str, name: &str, default: Self::Value);
}

pub trait IniDefaultReadable {
    ///
    /// @brief Reads in values from the INI file using a default configuraion.
    /// @param ini The INI to read from.
    ///
    fn read_ini_default(&mut self, ini: &Ini);
}

/// Configures an INI section with default values
pub struct DefaultIniSection<T: IniNamedReadable> {
    field: T,
    section: &'static str,
    default: <T as IniNamedReadable>::Value
}

/// Configures an INI field with default values.
pub struct DefaultIniField<T: IniUnnamedReadable> {
    field: T,
    section: &'static str,
    name: &'static str,
    default: <T as IniUnnamedReadable>::Value
}

impl<T: IniNamedReadable + Default> DefaultIniSection<T> {
    /// Creates a new INI field with default presets.
    pub fn new(
        section: &'static str,
        default: <T as IniNamedReadable>::Value
    ) -> Self {
        Self {
            field: T::default(),
            section,
            default
        }
    }
}

impl<T: IniNamedReadable> IniDefaultReadable for DefaultIniSection<T> {
    fn read_ini_default(
        &mut self,
        ini: &Ini
    ) {
        self.field.read_ini_named(ini, self.section, self.default);
    }
}

impl<T: IniNamedReadable> Deref for DefaultIniSection<T> {
    type Target = T;
    fn deref(
        &self
    ) -> &Self::Target {
        &self.field
    }
}

impl<T: IniUnnamedReadable + Default> DefaultIniField<T> {
    /// Creates a new INI field with default presets.
    pub fn new(
        section: &'static str,
        name: &'static str,
        default: <T as IniUnnamedReadable>::Value
    ) -> Self {
        Self {
            field: T::default(),
            section,
            name,
            default
        }
    }
}

impl<T: IniUnnamedReadable> IniDefaultReadable for DefaultIniField<T> {
    fn read_ini_default(
        &mut self,
        ini: &Ini
    ) {
        self.field.read_ini_unnamed(ini, self.section, self.name, self.default);
    }
}

impl<T: IniUnnamedReadable> Deref for DefaultIniField<T> {
    type Target = T;
    fn deref(
        &self
    ) -> &Self::Target {
        &self.field
    }
}
