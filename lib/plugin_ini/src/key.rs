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
use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::mem::size_of;
use std::fmt;

/// Borrowed version of a key string.
#[repr(transparent)]
pub struct KeyStr(str);

/// An INI key string. Comparison is case insensitive.
#[derive(Clone)]
#[repr(transparent)]
pub struct KeyString(Rc<String>);

impl KeyStr {
    /// Creates a KeyStr from a string.
    pub const fn new<'a>(
        s: &'a str
    ) -> &'a Self {
        assert!(size_of::<&Self>() == size_of::<&str>());

        unsafe {
            // SAFETY: KeyStr is declared as transparent.
            &*(s as *const str as *const Self)
        }
    }

    /// Gets the underlying str, preserving the original case.
    pub const fn get(
        &self
    ) -> &str {
        &self.0
    }
}

impl fmt::Display for KeyStr {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>
    ) -> Result<(), fmt::Error> {
        write!(f, "{}", self.get())
    }
}

impl<T: Borrow<str> + ?Sized> PartialEq<T> for KeyStr {
    #[cfg(not(feature = "unicode_keys"))]
    fn eq(
        &self,
        rhs: &T
    ) -> bool {
        let lhs = self.get().as_bytes();
        let rhs = rhs.borrow().as_bytes();

        if lhs.len() != rhs.len() {
            return false;
        }

        for i in 0..lhs.len() {
            if lhs[i].to_ascii_lowercase() != rhs[i].to_ascii_lowercase() {
                return false;
            }
        }

        return true;
    }

    #[cfg(feature = "unicode_keys")]
    fn eq(
        &self,
        rhs: &T
    ) -> bool {
        let lhs = self.get();
        let rhs = rhs.borrow();

        let (mut lhs_chars, mut rhs_chars) = (lhs.chars(), rhs.chars());
        while let Some(l) = lhs_chars.next() {
            let r = rhs_chars.next();
            if r.is_none() {
                return false;
            }
            let r = r.unwrap();

            let (mut llc, mut lrc) = (l.to_lowercase(), r.to_lowercase());
            while let Some(ll) = llc.next() {
                let lr = lrc.next();
                if lr.is_none() {
                    return false;
                }
                let lr = lr.unwrap();

                if ll != lr {
                    return false;
                }
            }

            if lrc.next().is_some() {
                return false;
            }
        }

        return rhs_chars.next().is_none();
    }
}

impl PartialEq<KeyStr> for KeyStr {
    fn eq(
        &self,
        rhs: &KeyStr
    ) -> bool {
        self == rhs.get()
    }
}

impl Eq for KeyStr {}

impl Hash for KeyStr {
    #[cfg(not(feature = "unicode_keys"))]
    fn hash<H: Hasher>(
        &self,
        state: &mut H
    ) {
        for c in self.get().as_bytes() {
            state.write_u8(c.to_ascii_lowercase());
        }
    }

    #[cfg(feature = "unicode_keys")]
    fn hash<H: Hasher>(
        &self,
        state: &mut H
    ) {
        for c in self.get().chars() {
            for l in c.to_lowercase() {
                let mut utf8: [u8; size_of::<char>()] = [0; size_of::<char>()];
                state.write(l.encode_utf8(&mut utf8).as_bytes());
            }
        }
    }
}

impl KeyString {
    /// Creates a new key string.
    pub fn new(
        s: String
    ) -> Self {
        Self(Rc::new(s))
    }

    /// Borrows the key string as a KeyStr.
    pub fn as_key_str(
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

impl fmt::Display for KeyString {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>
    ) -> Result<(), fmt::Error> {
        write!(f, "{}", self.as_key_str())
    }
}

impl PartialEq for KeyString {
    fn eq(
        &self,
        rhs: &Self
    ) -> bool {
        self.as_key_str() == rhs.as_key_str()
    }
}

impl<T: Borrow<str> + ?Sized> PartialEq<T> for KeyString {
    fn eq(
        &self,
        rhs: &T
    ) -> bool {
        self.as_key_str() == KeyStr::new(rhs.borrow())
    }
}

impl Eq for KeyString {}

impl Hash for KeyString {
    fn hash<H: Hasher>(
        &self,
        state: &mut H
    ) {
        self.as_key_str().hash(state);
    }
}
