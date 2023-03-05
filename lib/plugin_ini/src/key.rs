//!
//! @file key.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Key value for INI hash maps. Case insensitive.
//! @bug No known bugs.
//!
//! Note that the key is implemented behind an RC, to allow it to be stored in both a vector
//! and a hash map (for O(1) ordering) without reallocation. Another option would be to store
//! an order mark with the value in a normal hash map, but that would require the iterator
//! of the INI file to allocate.
//!

use std::rc::Rc;
use std::ops::{Borrow, Deref};
use std::hash::{Hash, Hasher};
use std::mem::size_of;
use std::fmt;

/// Borrowed version of a key string.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct KeyStr<'a>(&'a str);

/// An INI key string. Comparison is case insensitive.
#[derive(Clone)]
pub struct KeyString(Rc<String>);

impl<'a> KeyStr<'a> {
    /// Creates a KeyStr from a string.
    pub fn new(
        s: &'a str
    ) -> Self {
        Self(s)
    }

    /// Gets the underlying str, preserving the original case.
    pub fn get(
        &self
    ) -> &str {
        self.0
    }
}

impl<T: Borrow<&str>> PartialEq<T> for KeyStr {
    fn eq(
        &self,
        rhs: &T
    ) -> bool {
        let lhs = self.0;
        let rhs = rhs.borrow();

        let mut iter = lhs.chars().zip(rhs.chars());
        for (l, r) in iter {
            if l.to_lowercase() != r.to_lowercase() {
                return false;
            }
        }

        let (li, ri) = iter.unzip();
        return li.next().is_none() && ri.next().is_none();
    }
}

impl<T: Borrow<&str>> Eq<T> for KeyStr {}

impl KeyString {
    /// Borrows the key string as a KeyStr.
    pub fn as_key_str(
        &'a self
    ) -> KeyStr<'a> {
        KeyStr(self.0.borrow().as_str())
    }

    /// Gets the underlying string, preserving the original case.
    pub fn get(
        &self
    ) -> String {
        self.0.borrow().clone()
    }
}

impl fmt::Display for KeyString {
    fn fmt(
        &self,
        &mut fmt::Formatter<'_>
    ) -> Result<(), fmt::Error> {
        write!("{}", self.0.borrow())?;
    }
}

impl<T: Borrow<&str>> PartialEq<T> for KeyString {
    fn eq(
        &self,
        rhs: &T
    ) -> bool {
        self.as_key_str() == KeyStr::new(rhs.borrow())
    }
}

impl<T: Borrow<&str>> Eq<T> for KeyString {}

impl Hash for KeyString {
    fn hash<H: Hasher>(
        &self,
        state: &mut H
    ) {
        for c in self.0.borrow().chars() {
            let c = c.to_lowercase();
            let mut utf8: [u8; size_of::<char>()] = [0; size_of::<char>()];
            state.write(c.encode_utf8(&mut utf8).as_bytes());
        }
    }
}

