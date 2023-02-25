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

/// The metadata associated with each section in the INI file.
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

/// An iterator over the sections in an INI (in order).
pub struct SectionIter<'a>(Vec<(&'a String, &'a SectionMeta)>);

/// Manages an INI file, allowing it to be updated, read, and written to a file.
pub struct Ini {
    file_comment: Option<String>,
    sections: HashMap<String, SectionMeta>,
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

    /// Writes the contents of the INI object to the given file.
    pub fn write_file(
        &self,
        path: &Path
    ) -> Result<(), std::io::Error> {
        let mut f = File::create(path)?;

        // Load in and sort the sections by their sequence number, to ensure ordering in the file
        // is preserved.
        let sections = {
            let mut v = Vec::new();
            for section in self.sections.iter() { v.push(section); }
            v.sort_by(|lhs, rhs| {
                lhs.1.seq.unwrap_or(usize::MAX).cmp(&rhs.1.seq.unwrap_or(usize::MAX))
            });
            v
        };

        if let Some(ref file_comment) = self.file_comment {
            write!(&mut f, "{}", file_comment)?;
        }

        for (name, meta) in sections.iter() {
            if let Some(ref pre) = meta.prefix { write!(&mut f, "{}", pre)?; }
            write!(&mut f, "[{}]", name)?;
            if let Some(ref comment) = meta.inline_comment { write!(&mut f, " {}", comment)?; }

            // Order the fields, for the same reason as the sections.
            let fields = {
                let mut v = Vec::new();
                for field in meta.fields.iter() { v.push(field); }
                v.sort_by(|lhs, rhs| {
                    lhs.1.seq.unwrap_or(usize::MAX).cmp(&rhs.1.seq.unwrap_or(usize::MAX))
                });
                v
            };

            for (name, meta) in fields.iter() {
                if let Some(ref pre) = meta.prefix { write!(&mut f, "{}", pre)?; }
                write!(&mut f, "{}", name)?;
                if let Some(ref val) = meta.val { write!(&mut f, " = {}", val)?; }
                if let Some(ref comment) = meta.inline_comment { write!(&mut f, " {}", comment)?; }
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
