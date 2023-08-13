//!
//! @file lib.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Ini file loader and updating interface implementation.
//! @bug No known bugs.
//!

#![no_std]
extern crate alloc;

use core::str::FromStr;
use core::ffi::CStr;
use core::cmp::Ordering;
use core::borrow::Borrow;
use core::mem::size_of;
use core::fmt::Write;
use alloc::rc::Rc;
use alloc::vec::Vec;
use alloc::string::String;

use cstdio::File;

////////////////////////////////////////////////////////////////////////////////////////////////////
// INI Implementation
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Characters which can be used to begin an inline comment.
const COMMENT_CHARS: &[char] = &['#', ';'];

/// Manages an INI file, allowing it to be updated, read, and written to a file.
pub struct Ini {
    sections: IniMap<SectionMeta>,
    suffix: Option<String>
}

/// A section in the INI file.
pub struct Section<'a> {
    section: &'a str,
    meta: &'a SectionMeta
}

/// A field in the INI file.
pub struct Field<'a> {
    field: &'a str,
    meta: &'a FieldMeta
}

///
/// An iterator over the sections in an INI (in order).
///
/// Sections merged from another file will appear at the end of the iteration in an undefined
/// order.
///
pub struct SectionIter<'a>(IniMapIter<'a, SectionMeta>);

///
/// An iterator over each field within a section (in order).
///
/// Fields merged from another file will appear at the end of the iteration in an undefined order.
///
pub struct FieldIter<'a>(IniMapIter<'a, FieldMeta>);

////////////////////////////////////////////////////////////////////////////////////////////////////

/// The metadata associated with each section in the INI file.
#[derive(Clone)]
struct SectionMeta {
    prefix: Option<String>,
    inline_comment: Option<String>,
    fields: IniMap<FieldMeta>
}

/// The metadata associated with each field in the INI file.
#[derive(Clone)]
struct FieldMeta {
    prefix: Option<String>,
    inline_comment: Option<String>,
    val: Option<String>
}

////////////////////////////////////////////////////////////////////////////////////////////////////

impl Ini {
    /// Loads in an INI file from the given path.
    pub fn from_path(
        path: &CStr
    ) -> Result<Self, ()> {
        let mut ret = Ini { sections: IniMap { order: Vec::new(), map: Vec::new() }, suffix: None };
        ret.load_path(path)?;
        Ok(ret)
    }

    /// Loads in an INI file from the given string.
    pub fn from_str(
        s: &str
    ) -> Result<Self, ()> {
        let mut ret = Ini { sections: IniMap { order: Vec::new(), map: Vec::new() }, suffix: None };
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
    ///
    /// Returns true if any updates were performed, and false otherwise.
    pub fn update(
        &mut self,
        delta: &Self
    ) -> bool {
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

        return changed;
    }

    /// Renames any fields/sections found within the first element of a tuple to the second element.
    ///
    /// Returns true if any substitutions were made, and false otherwise.
    pub fn rename(
        &mut self,
        sub: &[(&str, &str)]
    ) -> bool {
        let mut changed = self.sections.rename(sub);
        for section in self.sections.order.iter_mut() {
            changed = changed || section.1.fields.rename(sub);
        }
        return changed;
    }

    /// Writes the contents of the INI object to the given file.
    pub fn write_file(
        &self,
        path: &CStr
    ) -> Result<(), core::fmt::Error> {
        let mut f = File::open(path, core_util::cstr!("w+")).map_err(|_| core::fmt::Error)?;

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

    /// Loads a configuration in from the given file.
    fn load_path(
        &mut self,
        path: &CStr
    ) -> Result<(), ()> {
        self.load_str(&File::open(path, core_util::cstr!("r"))?.into_string()?)
    }

    /// Loads a configuration in from the given string.
    fn load_str(
        &mut self,
        conf: &str
    ) -> Result<(), ()> {
        let is_whitespace = |l: &str| { l.trim().len() == 0 };
        let is_comment = |l: &str| { l.trim().starts_with(COMMENT_CHARS) };

        let mut section = None;
        let mut s = String::new();

        for line in conf.lines() {
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
                fields: IniMap { order: Vec::new(), map: Vec::new() }
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

impl<'a> Iterator for SectionIter<'a> {
    type Item = Section<'a>;
    fn next(
        &mut self
    ) -> Option<Self::Item> {
        self.0.next().map(|i| Section { section: i.0.get(), meta: i.1 })
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

////////////////////////////////////////////////////////////////////////////////////////////////////
// INI ordered map implementation
////////////////////////////////////////////////////////////////////////////////////////////////////
// In order to prevent an unnecessary lookups during iteration, this implementation
// stores indexes as the values in a searchable mapping table. This adds a lair of indirection to
// look-ups, but the cost savings in iteration is worth it for our use case. A binary search
// table is used instead of a hash map, as it provides comparable performance while also
// generating a smaller specialization.

/// A map which maintains a strict ordering on the keys it contains.
#[derive(Clone)]
struct IniMap<V> {
    order: Vec<(KeyString, V)>,
    map: Vec<(KeyString, usize)>,
}

/// Iterates over the elements in an ordered map, in order.
struct IniMapIter<'a, V> {
    order: &'a Vec<(KeyString, V)>,
    index: usize
}

////////////////////////////////////////////////////////////////////////////////////////////////////

impl<V> IniMap<V> {
    /// Gets the element with the given key.
    fn get<'a>(
        &'a self,
        key: &str
    ) -> Option<&'a V> {
        self.search(key).ok().map(|i| &self.order[i].1)
    }

    /// Gets a mutable reference to the element with the given key.
    fn get_mut<'a>(
        &'a mut self,
        key: &str
    ) -> Option<&'a mut V> {
        self.search(key).ok().map(|i| &mut self.order[i].1)
    }

    /// Gets the (key, value) associated with the given key.
    fn get_key_value(
        &self,
        key: &str
    ) -> Option<(&KeyString, &V)> {
        if let Ok(i) = self.search(key) {
            Some((&self.order[i].0, &self.order[i].1))
        } else {
            None
        }
    }

    /// Inserts a new (key, val) into the map. Values are ordered based on their insertion order.
    fn insert(
        &mut self,
        key: String,
        val: V
    ) {
        let key = KeyString::new(key);
        match self.search(key.as_key_str().get()) {
            Ok(i) => {
                self.order[i].1 = val;
            },
            Err(i) => {
                self.map.insert(i, (key.clone(), self.order.len()));
                self.order.push((key, val));
            }
        }
    }

    /// Renames a key in the invoking map based on the given substitution tuples.
    ///
    /// Returns true if any key was renamed, and false otherwise.
    fn rename(
        &mut self,
        sub: &[(&str, &str)]
    ) -> bool {
        let mut changed = false;
        for group in sub.iter() {
            let group = (KeyStr::new(group.0), KeyStr::new(group.1));
            if let Ok(i) = self.map.binary_search_by(|lhs| lhs.0.as_key_str().cmp(group.0)) {
                changed = true;
                let new_key = KeyString::new(String::from_str(group.1.get()).unwrap());
                let order_index = self.map[i].1;

                self.map.remove(i);
                self.map.insert(i, (new_key.clone(), order_index));
                self.order[order_index].0 = new_key;
            }
        }

        return changed;
    }

    /// Gets an iterator for this map.
    fn iter(
        &self
    ) -> IniMapIter<'_, V> {
        IniMapIter {
            order: &self.order,
            index: 0
        }
    }

    /// Binary searches for the given string in the map.
    fn search(
        &self,
        key: &str
    ) -> Result<usize, usize> {
        let key = KeyStr::new(key);
        self.map.binary_search_by(|k| k.0.as_key_str().cmp(key)).map(|i| self.map[i].1)
    }
}

impl<'a, V> Iterator for IniMapIter<'a, V> {
    type Item = (&'a KeyStr, &'a V);
    fn next(
        &mut self
    ) -> Option<Self::Item> {
        if self.index < self.order.len() {
            let ret = (self.order[self.index].0.as_key_str(), &self.order[self.index].1);
            self.index += 1;
            Some(ret)
        } else {
            None
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Case-insensitive keystring implementation
////////////////////////////////////////////////////////////////////////////////////////////////////
//
// Note that the key is implemented behind an RC, to allow it to be stored in both a vector
// and a hash map (for O(1) ordering) without reallocation. Another option would be to store
// an order mark with the value in a normal hash map, but that would require the iterator
// of the INI file to allocate.

/// Borrowed version of a key string.
#[repr(transparent)]
struct KeyStr(str);

/// An INI key string. Comparison is case insensitive.
#[derive(Clone)]
#[repr(transparent)]
struct KeyString(Rc<String>);

////////////////////////////////////////////////////////////////////////////////////////////////////

impl KeyStr {
    /// Creates a KeyStr from a string.
    const fn new<'a>(
        s: &'a str
    ) -> &'a Self {
        assert!(size_of::<&Self>() == size_of::<&str>());

        unsafe {
            // SAFETY: KeyStr is declared as transparent.
            &*(s as *const str as *const Self)
        }
    }

    /// Gets the underlying str, preserving the original case.
    const fn get(
        &self
    ) -> &str {
        &self.0
    }

    /// Compares two key strs.
    fn compare(
        &self,
        rhs: &Self
    ) -> Ordering {
        let lhs = self.get().as_bytes();
        let rhs = rhs.get().as_bytes();
        for i in 0..core::cmp::min(lhs.len(), rhs.len()) {
            let res = lhs[i].to_ascii_lowercase().cmp(&rhs[i].to_ascii_lowercase());
            if let Ordering::Equal = res {
                continue;
            } else {
                return res;
            }
        }

        lhs.len().cmp(&rhs.len())
    }
}

impl PartialEq<KeyStr> for KeyStr {
    fn eq(
        &self,
        rhs: &KeyStr
    ) -> bool {
        self.compare(rhs) == Ordering::Equal
    }
}


impl Eq for KeyStr {}

impl PartialOrd<KeyStr> for KeyStr {
    fn partial_cmp(
        &self,
        rhs: &KeyStr
    ) -> Option<Ordering> {
        Some(self.compare(rhs))
    }
}

impl Ord for KeyStr {
    fn cmp(
        &self,
        rhs: &Self
    ) -> Ordering {
        self.compare(rhs)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

impl KeyString {
    /// Creates a new key string.
    fn new(
        s: String
    ) -> Self {
        Self(Rc::new(s))
    }

    /// Borrows the key string as a KeyStr.
    fn as_key_str(
        &self
    ) -> &KeyStr {
        KeyStr::new(<Rc<String> as Borrow<String>>::borrow(&self.0).as_str())
    }
}

impl Borrow<KeyStr> for KeyString {
    fn borrow(
        &self
    ) -> &KeyStr {
        self.as_key_str()
    }
}

impl<T: Borrow<KeyStr> + ?Sized> PartialEq<T> for KeyString {
    fn eq(
        &self,
        rhs: &T
    ) -> bool {
        self.as_key_str() == rhs.borrow()
    }
}

impl Eq for KeyString {}

impl<T: Borrow<KeyStr> + ?Sized> PartialOrd<T> for KeyString {
    fn partial_cmp(
        &self,
        rhs: &T
    ) -> Option<Ordering> {
        Some(self.as_key_str().cmp(rhs.borrow()))
    }
}

impl Ord for KeyString {
    fn cmp(
        &self,
        rhs: &Self
    ) -> Ordering {
        self.partial_cmp(rhs).unwrap()
    }
}
