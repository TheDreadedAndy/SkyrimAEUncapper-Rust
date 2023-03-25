//!
//! @file map.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Ordered, case-insensitive, map implementation for strings.
//! @bug No known bugs.
//!
//! In order to prevent an unnecessary lookups during iteration, this implementation
//! stores indexes as the values in a searchable mapping table. This adds a lair of indirection to
//! look-ups, but the cost savings in iteration is worth it for our use case. A binary search
//! table is used instead of a hash map, as it provides comparable performance while also
//! generating a smaller specialization.
//!

use std::vec::Vec;

use crate::key::*;

/// A map which maintains a strict ordering on the keys it contains.
#[derive(Clone)]
pub struct IniMap<V> {
    order: Vec<(KeyString, V)>,
    map: Vec<(KeyString, usize)>,
}

/// Iterates over the elements in an ordered map, in order.
pub struct IniMapIter<'a, V> {
    order: &'a Vec<(KeyString, V)>,
    index: usize
}

impl<V> IniMap<V> {
    /// Creates a new ordered map.
    pub fn new() -> Self {
        Self {
            order: Vec::new(),
            map: Vec::new()
        }
    }

    /// Gets the element with the given key.
    pub fn get<'a>(
        &'a self,
        key: &str
    ) -> Option<&'a V> {
        self.search(key).ok().map(|i| &self.order[i].1)
    }

    /// Gets a mutable reference to the element with the given key.
    pub fn get_mut<'a>(
        &'a mut self,
        key: &str
    ) -> Option<&'a mut V> {
        self.search(key).ok().map(|i| &mut self.order[i].1)
    }

    /// Gets the (key, value) associated with the given key.
    pub fn get_key_value(
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
    pub fn insert(
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

    /// Gets an iterator for this map.
    pub fn iter(
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
