//!
//! @file ini.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Implementation of INI interface.
//! @bug No known bugs.
//!

use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};
use std::str::FromStr;

use crate::map::*;

/// Characters which can be used to begin an inline comment.
const COMMENT_CHARS: &[char] = &['#', ';'];

/// The metadata associated with each field in the INI file.
#[derive(Clone)]
struct FieldMeta {
    prefix: Option<String>,
    inline_comment: Option<String>,
    val: Option<String>
}

/// A field in the INI file.
pub struct Field<'a> {
    field: &'a str,
    meta: &'a FieldMeta
}

///
/// An iterator over each field within a section (in order).
///
/// Fields merged from another file will appear at the end of the iteration in an undefined order.
///
pub struct FieldIter<'a>(IniMapIter<'a, FieldMeta>);

/// The metadata associated with each section in the INI file.
#[derive(Clone)]
struct SectionMeta {
    prefix: Option<String>,
    inline_comment: Option<String>,
    fields: IniMap<FieldMeta>
}

/// A section in the INI file.
pub struct Section<'a> {
    section: &'a str,
    meta: &'a SectionMeta
}

///
/// An iterator over the sections in an INI (in order).
///
/// Sections merged from another file will appear at the end of the iteration in an undefined
/// order.
///
pub struct SectionIter<'a>(IniMapIter<'a, SectionMeta>);

/// Manages an INI file, allowing it to be updated, read, and written to a file.
pub struct Ini {
    file_comment: Option<String>,
    sections: IniMap<SectionMeta>,
    suffix: Option<String>
}

impl<'a> Field<'a> {
    /// Gets the name of the given field.
    pub fn name(
        &self
    ) -> &str {
        self.field
    }

    /// Attempts to read the value for the given field.
    pub fn value<T: FromStr>(
        &self
    ) -> Option<T> {
        self.meta.val.as_ref().and_then(|s| T::from_str(s).ok())
    }
}

impl<'a> Iterator for FieldIter<'a> {
    type Item = Field<'a>;
    fn next(
        &mut self
    ) -> Option<Self::Item> {
        self.0.next().map(|i| Field { field: i.0.get(), meta: i.1 })
    }
}

impl<'a> Section<'a> {
    /// Gets the name of the given section.
    pub fn name(
        &self
    ) -> &str {
        self.section
    }

    /// Gets a field in the given section.
    pub fn field(
        &self,
        name: &str
    ) -> Result<Field<'a>, ()> {
        let (field, meta) = self.meta.fields.get_key_value(name).ok_or(())?;
        Ok(Field { field: field.as_key_str().get(), meta })
    }

    /// Gets an iterator over all the fields in a section.
    pub fn fields(
        &self
    ) -> FieldIter<'a> {
        FieldIter(self.meta.fields.iter())
    }
}

impl<'a> Iterator for SectionIter<'a> {
    type Item = Section<'a>;
    fn next(
        &mut self
    ) -> Option<Self::Item> {
        self.0.next().map(|i| Section { section: i.0.get(), meta: i.1 })
    }
}

impl Ini {
    /// Loads in an INI file from the given path.
    pub fn from_path(
        path: &Path
    ) -> Result<Self, ()> {
        let mut ret = Self::new();
        ret.load_path(path)?;
        Ok(ret)
    }

    /// Loads in an INI file from the given string.
    pub fn from_str(
        s: &str
    ) -> Result<Self, ()> {
        let mut ret = Self::new();
        ret.load_str(s)?;
        Ok(ret)
    }

    /// Gets a single section within the INI file.
    pub fn section<'a>(
        &'a self,
        section: &str
    ) -> Result<Section<'a>, ()> {
        let (section, meta) = self.sections.get_key_value(section).ok_or(())?;
        Ok(Section { section: section.as_key_str().get(), meta })
    }

    /// Gets an iterator over all sections in the INI file.
    pub fn sections<'a>(
        &'a self
    ) -> SectionIter<'a> {
        SectionIter(self.sections.iter())
    }

    /// Gets a value in the INI from the given section/field pair.
    pub fn get<T: FromStr>(
        &self,
        section: &str,
        field: &str
    ) -> Option<T> {
        self.section(section).ok()?.field(field).ok()?.value()
    }

    /// Updates the sections/fields in the given map with values found only in the second map.
    pub fn update(
        &mut self,
        delta: &Self
    ) -> Option<()> {
        let mut changed = false;

        for delta_section in delta.sections() {
            if self.sections.get(delta_section.name()).is_none() {
                let name = String::from_str(delta_section.name()).unwrap();
                let meta = delta_section.meta.clone();
                self.sections.insert(name, meta);
                changed = true;
                continue;
            }

            let section = self.sections.get_mut(delta_section.name()).unwrap();
            for delta_field in delta_section.fields() {
                if section.fields.get(delta_field.name()).is_none() {
                    let name = String::from_str(delta_field.name()).unwrap();
                    let meta = delta_field.meta.clone();
                    section.fields.insert(name, meta);
                    changed = true;
                }
            }
        }

        if changed { Some(()) } else { None }
    }

    /// Writes the contents of the INI object to the given file.
    pub fn write_file(
        &self,
        path: &Path
    ) -> Result<(), std::io::Error> {
        let mut f = File::create(path)?;

        if let Some(ref file_comment) = self.file_comment {
            write!(&mut f, "{}", file_comment)?;
        }

        for section in self.sections() {
            if let Some(ref pre) = section.meta.prefix { write!(&mut f, "{}", pre)?; }
            write!(&mut f, "[{}]", section.name())?;
            if let Some(ref comment) = section.meta.inline_comment {
                write!(&mut f, " #{}", comment)?;
            }
            write!(&mut f, "\n")?;

            for Field { field, meta } in section.fields() {
                if let Some(ref pre) = meta.prefix { write!(&mut f, "{}", pre)?; }
                write!(&mut f, "{}", field)?;
                if let Some(ref val) = meta.val { write!(&mut f, " = {}", val)?; }
                if let Some(ref comment) = meta.inline_comment { write!(&mut f, " #{}", comment)?; }
                write!(&mut f, "\n")?;
            }
        }

        if let Some(ref suffix) = self.suffix {
            write!(&mut f, "{}", suffix)?;
        }

        Ok(())
    }

    /// Creates a new, empty, INI object.
    fn new() -> Self {
        Self {
            file_comment: None,
            sections: IniMap::new(),
            suffix: None
        }
    }

    /// Loads a configuration in from the given file.
    fn load_path(
        &mut self,
        path: &Path
    ) -> Result<(), ()> {
        let mut f = File::open(path).map_err(|_| ())?;
        let mut s = String::new();
        f.read_to_string(&mut s).map_err(|_| ())?;
        self.load_str(&s)
    }

    /// Loads a configuration in from the given string.
    fn load_str(
        &mut self,
        conf: &str
    ) -> Result<(), ()> {
        let is_whitespace = |l: &str| { l.trim().len() == 0 };
        let is_comment = |l: &str| { l.trim().starts_with(COMMENT_CHARS) };

        let mut first_comment = true;
        let mut section = None;
        let mut s = String::new();

        for line in conf.lines() {
            // Check if we are still parsing the file comment.
            if first_comment {
                if is_comment(line) {
                    s += line;
                    s += "\n";
                    continue;
                } else {
                    first_comment = false;
                    self.file_comment = Some(s);
                    s = String::new();
                }
            }

            // Otherwise, determine what this line is a part of.
            if is_comment(line) || is_whitespace(line) {
                s += line;
                s += "\n";
            } else if line.trim().starts_with('[') {
                section = Some(self.define_section(s, line)?);
                s = String::new();
            } else {
                self.define_field(section.as_ref().ok_or(())?, s, line)?;
                s = String::new();
            }
        }

        // Save any trailing data in the file.
        if s.len() > 0 {
            self.suffix = Some(s);
        }

        Ok(())
    }

    ///
    /// Adds or gets the section with the given name from the INI file.
    ///
    /// If the given section is already in the file, then the given prefix is lost.
    ///
    fn define_section(
        &mut self,
        prefix: String,
        line: &str
    ) -> Result<String, ()> {
        let prefix = if prefix.len() > 0 { Some(prefix) } else { None };
        let (line, comment) = self.split_comment(line);

        if !line.starts_with('[') || !line.ends_with(']') {
            return Err(());
        }

        let name = line.split_at(line.len() - 1).0.split_at(1).1;
        if self.sections.get(name).is_none() {
            let section = String::from_str(name).unwrap();
            let meta = SectionMeta {
                prefix,
                inline_comment: comment.map(|s| String::from_str(s).unwrap()),
                fields: IniMap::new()
            };
            self.sections.insert(section, meta);
        }

        Ok(String::from_str(name).unwrap())
    }

    ///
    /// Adds a field to the given section in the INI file.
    ///
    /// If the given field is already in the file, then the given prefix is lost
    /// and the new value is ignored.
    ///
    fn define_field(
        &mut self,
        section: &str,
        prefix: String,
        line: &str
    ) -> Result<(), ()> {
        let prefix = if prefix.len() > 0 { Some(prefix) } else { None };
        let (line, comment) = self.split_comment(line);
        let (key, val) = if let Some((k, v)) = line.split_once('=') {
            (k.trim(), Some(v.trim()))
        } else {
            (line.trim(), None)
        };

        let section = self.sections.get_mut(section).ok_or(())?;
        if let None = section.fields.get(key) {
            let key = String::from_str(key).unwrap();
            section.fields.insert(key, FieldMeta {
                prefix,
                inline_comment: comment.map(|s| String::from_str(s).unwrap()),
                val: val.map(|s| String::from_str(s).unwrap())
            });

            Ok(())
        } else {
            Err(())
        }
    }

    /// Splits off the inline comment from a line of text.
    fn split_comment<'a>(
        &self,
        line: &'a str
    ) -> (&'a str, Option<&'a str>) {
        if let Some((l, c)) = line.split_once(COMMENT_CHARS) {
            (l.trim(), Some(c))
        } else {
            (line.trim(), None)
        }
    }
}
