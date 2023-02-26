//!
//! @file ini.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Implementation of INI interface.
//! @bug No known bugs.
//!

use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};
use std::str::FromStr;

/// The metadata associated with each field in the INI file.
#[derive(Clone)]
struct FieldMeta {
    seq: Option<usize>,
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
pub struct FieldIter<'a> {
    index: usize,
    data: Vec<(&'a String, &'a FieldMeta)>
}

/// The metadata associated with each section in the INI file.
#[derive(Clone)]
struct SectionMeta {
    seq: Option<usize>,
    prefix: Option<String>,
    inline_comment: Option<String>,
    fields: HashMap<String, FieldMeta>
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
pub struct SectionIter<'a> {
    index: usize,
    data: Vec<(&'a String, &'a SectionMeta)>
}

/// Manages an INI file, allowing it to be updated, read, and written to a file.
pub struct Ini {
    file_comment: Option<String>,
    sections: HashMap<String, SectionMeta>,
    suffix: Option<String>,
    comment_chars: Vec<char>
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
        if self.index < self.data.len() {
            let ret = Field { field: self.data[self.index].0, meta: self.data[self.index].1 };
            self.index += 1;
            Some(ret)
        } else {
            None
        }
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
        Ok(Field { field, meta })
    }

    /// Gets an iterator over all the fields in a section.
    pub fn fields(
        &self
    ) -> FieldIter<'a> {
        let data = {
            let mut v = Vec::new();
            for field in self.meta.fields.iter() { v.push(field); }
            v.sort_by(|lhs, rhs| {
                lhs.1.seq.unwrap_or(usize::MAX).cmp(&rhs.1.seq.unwrap_or(usize::MAX))
            });
            v
        };

        FieldIter {
            index: 0,
            data
        }
    }
}

impl<'a> Iterator for SectionIter<'a> {
    type Item = Section<'a>;
    fn next(
        &mut self
    ) -> Option<Self::Item> {
        if self.index < self.data.len() {
            let ret = Section { section: self.data[self.index].0, meta: self.data[self.index].1 };
            self.index += 1;
            Some(ret)
        } else {
            None
        }
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

    /// Sets the line comment start characters for the next INI file loaded into this structure.
    pub fn comment_chars(
        &mut self,
        pat: Vec<char>
    ) {
        self.comment_chars = pat;
    }

    /// Gets a single section within the INI file.
    pub fn section<'a>(
        &'a self,
        section: &str
    ) -> Result<Section<'a>, ()> {
        let (section, meta) = self.sections.get_key_value(section).ok_or(())?;
        Ok(Section { section, meta })
    }

    /// Gets an iterator over all sections in the INI file.
    pub fn sections<'a>(
        &'a self
    ) -> SectionIter<'a> {
        let data = {
            let mut v = Vec::new();
            for section in self.sections.iter() { v.push(section); }
            v.sort_by(|lhs, rhs| {
                lhs.1.seq.unwrap_or(usize::MAX).cmp(&rhs.1.seq.unwrap_or(usize::MAX))
            });
            v
        };

        SectionIter {
            index: 0,
            data
        }
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
                let mut meta = delta_section.meta.clone();
                meta.seq = None;
                assert!(self.sections.insert(name, meta).is_none());
                changed = true;
                continue;
            }

            let section = self.sections.get_mut(delta_section.name()).unwrap();
            for delta_field in delta_section.fields() {
                if section.fields.get(delta_field.name()).is_none() {
                    let name = String::from_str(delta_field.name()).unwrap();
                    let mut meta = delta_field.meta.clone();
                    meta.seq = None;
                    assert!(section.fields.insert(name, meta).is_none());
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
            sections: HashMap::new(),
            suffix: None,
            comment_chars: vec!['#', ';']
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
        let is_comment = |l: &str, c: &[char]| { l.trim().starts_with(c)};

        let mut first_comment = true;
        let mut section = None;
        let mut s = String::new();
        let mut section_seq = 0;
        let mut field_seq = 0;

        for line in conf.lines() {
            // Check if we are still parsing the file comment.
            if first_comment {
                if is_comment(line, self.comment_chars.as_slice()) {
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
            if is_comment(line, self.comment_chars.as_slice()) || is_whitespace(line) {
                s += line;
                s += "\n";
            } else if line.trim().starts_with('[') {
                section = Some(self.define_section(section_seq, s, line)?);
                section_seq += 1;
                s = String::new();
            } else {
                self.define_field(section.as_ref().ok_or(())?, field_seq, s, line)?;
                field_seq += 1;
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
        seq: usize,
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
                seq: Some(seq),
                prefix,
                inline_comment: comment.map(|s| String::from_str(s).unwrap()),
                fields: HashMap::new()
            };
            assert!(self.sections.insert(section, meta).is_none());
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
        seq: usize,
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
            assert!(section.fields.insert(key, FieldMeta {
                seq: Some(seq),
                prefix,
                inline_comment: comment.map(|s| String::from_str(s).unwrap()),
                val: val.map(|s| String::from_str(s).unwrap())
            }).is_none());

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
        if let Some((l, c)) = line.split_once(self.comment_chars.as_slice()) {
            (l.trim(), Some(c.trim()))
        } else {
            (line.trim(), None)
        }
    }
}
