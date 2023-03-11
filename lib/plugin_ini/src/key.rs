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

use std::cmp::Ordering;
use std::rc::Rc;
use std::borrow::Borrow;
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

    /// Compares two key strs.
    #[cfg(not(feature = "unicode_keys"))]
    fn compare(
        &self,
        rhs: &Self
    ) -> Ordering {
        let lhs = self.get().as_bytes();
        let rhs = rhs.get().as_bytes();
        for i in 0..std::cmp::min(lhs.len(), rhs.len()) {
            let res = lhs[i].to_ascii_lowercase().cmp(&rhs[i].to_ascii_lowercase());
            if let Ordering::Equal = res {
                continue;
            } else {
                return res;
            }
        }

        lhs.len().cmp(&rhs.len())
    }
    #[cfg(feature = "unicode_keys")]
    fn compare(
        &self,
        rhs: &KeyStr
    ) -> Ordering {
        let lhs = self.get();
        let rhs = rhs.get();

        let (mut lhs_chars, mut rhs_chars) = (lhs.chars(), rhs.chars());
        while let Some(l) = lhs_chars.next() {
            let r = rhs_chars.next();
            if r.is_none() {
                return Ordering::Greater;
            }
            let r = r.unwrap();

            let (mut llc, mut lrc) = (l.to_lowercase(), r.to_lowercase());
            while let Some(ll) = llc.next() {
                let lr = lrc.next();
                if lr.is_none() {
                    return Ordering::Greater;
                }
                let lr = lr.unwrap();

                if ll != lr {
                    return ll.cmp(lr);
                }
            }

            if lrc.next().is_some() {
                return Ordering::Less;
            }
        }

        if rhs_chars.next().is_none() {
            Ordering::Equal
        } else {
            Ordering::Less
        }
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
