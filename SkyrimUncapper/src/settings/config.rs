//!
//! @file config.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Provides traits for reading in values from INI files.
//! @bug No known bugs.
//!

use ini::Ini;

pub trait IniReadable {
    /// @brief The type of the underlying values.
    type Value: Copy;

    ///
    /// @brief Reads the value of the config item from the given section of the given INI.
    /// @param ini The INI to read from.
    /// @param section The section of the INI to read from.
    /// @param default The default value to assume if none is available.
    ///
    fn read_ini(&mut self, ini: &Ini, section: &str, default: Self::Value);
}

pub trait IniDefaultReadable {
    ///
    /// @brief Reads in values from the INI file using a default configuraion.
    /// @param ini The INI to read from.
    ///
    fn read_ini_default(&mut self, ini: &Ini);
}

/// @brief Configures an INI field with default values
pub struct DefaultIniField<T: IniReadable> {
    field: T,
    section: &'static str,
    default: <T as IniReadable>::Value
}

impl<T: IniReadable> DefaultIniField<T> {
    /// @brief Creates a new INI field with default presets.
    pub fn new(
        field: T,
        section: &'static str,
        default: <T as IniReadable>::Value
    ) -> Self {
        Self {
            field,
            section,
            default
        }
    }
}

impl<T: IniReadable> IniDefaultReadable for DefaultIniField<T> {
    fn read_ini_default(
        &mut self,
        ini: &Ini
    ) {
        self.field.read_ini(ini, self.section, self.default);
    }
}
