//!
//! @file ini.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Implementation of INI interface.
//! @bug No known bugs.
//!

use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

use crate::order::OrderMap;

struct FieldMeta {
    prefix: Option<String>,
    inline_comment: Option<String>,
    val: Option<String>
}

pub struct Field<'a> {
    field: &'a str,
    meta: &'a FieldMeta
}

struct SectionMeta {
    prefix: Option<String>,
    inline_comment: Option<String>,
    fields: OrderMap<String, FieldMeta>
}

pub struct Section<'a> {
    section: &'a str,
    meta: &'a SectionMeta
}

pub struct Ini {
    file_comment: Option<String>,
    sections: OrderMap<String, SectionMeta>,
    suffix: Option<String>,
    comment_chars: Vec<char>
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

    /// Creates a new, empty, INI object.
    fn new() -> Self {
        Self {
            file_comment: None,
            sections: OrderMap::new(),
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
        let mut first_comment = true;
        let mut section = None;
        let mut s = String::new();
        let is_whitespace = |l: &str| { l.trim().len() == 0 };
        let is_comment = |l: &str| {
            l.trim().starts_with(self.comment_chars.as_slice())
        };

        for line in conf.lines() {
            // Check if we are still parsing the file comment.
            if first_comment {
                if is_comment(line) {
                    s += line;
                    continue;
                } else {
                    self.file_comment = Some(s);
                    s = String::new();
                }
            }

            // Otherwise, determine what this line is a part of.
            if is_comment(line) || is_whitespace(line) {
                s += line;
            } else if line.trim().starts_with('[') {
                section = Some(self.define_section(s, line)?);
                s = String::new();
            } else {
                section.ok_or(())?.define_field(s, line)?;
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
    fn define_section<'a>(
        &'a mut self,
        prefix: String,
        line: &str
    ) -> Result<&'a mut SectionMeta, ()> {
        let prefix = if prefix.len() > 0 { Some(prefix) } else { None };
        let (line, comment) = if let Some((l, c)) = line.split_once(self.comment_chars.as_slice()) {
            (l.trim(), Some(c.trim()))
        } else {
            (line.trim(), None)
        };

        if !line.starts_with('[') || !line.ends_with(']') {
            return Err(());
        }

        let name = line.split_at(line.len() - 1).0.split_at(1).1;
        if let Some(v) = self.sections.get_mut(name) {
            return Ok(v);
        }

        let section = String::from_str(name).unwrap();
        let meta = SectionMeta {
            prefix,
            inline_comment: comment.map(|s| String::from_str(s).unwrap()),
            fields: OrderMap::new()
        };
        assert!(self.sections.insert(self.sections.len() - 1, section, meta).is_none());
        Ok(self.sections.get_mut(name).ok_or(()).unwrap())
    }
}
