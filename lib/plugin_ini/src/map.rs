//!
//! @file map.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Ordered, case-insensitive, map implementation for strings.
//! @bug No known bugs.
//!

use std::vec::Vec;
use std::collections::HashMap;
use std::borrow::Borrow;

use crate::key::*;

/// A hash-map which maintains a strict ordering on the keys it contains.
#[derive(Clone)]
pub struct IniMap<V> {
    order: Vec<KeyString>,
    map: HashMap<KeyString, V>,
}

/// Iterates over the elements in an ordered map, in order.
pub struct IniMapIter<'a, V> {
    map: &'a IniMap<V>,
    index: usize
}

impl<V> IniMap<V> {
    /// Creates a new ordered hashmap.
    pub fn new() -> Self {
        Self {
            order: Vec::new(),
            map: HashMap::new()
        }
    }

    /// Gets the number of elements in the map.
    pub fn len(
        &self
    ) -> usize {
        self.order.len()
    }

    /// Gets the element with the given key.
    pub fn get<'a, Q>(
        &'a self,
        key: &Q
    ) -> Option<&'a V>
    where
        Q: Borrow<str> + ?Sized
    {
        self.map.get(&KeyStr::new(key.borrow()))
    }

    /// Gets a mutable reference to the element with the given key.
    pub fn get_mut<'a, Q>(
        &'a mut self,
        key: &Q
    ) -> Option<&'a mut V>
    where
        Q: Borrow<str> + ?Sized
    {
        self.map.get_mut(&KeyStr::new(key.borrow()))
    }

    /// Gets the key at the ith position in the map.
    pub fn get_key<'a>(
        &self,
        i: usize
    ) -> &KeyStr<'a> {
        <KeyString as Borrow<KeyStr<'a>>>::borrow(&self.order[i])
    }

    /// Gets the (key, value) associated with the given key.
    pub fn get_key_value<Q>(
        &self,
        key: &Q
    ) -> Option<(&KeyString, &V)>
    where
        Q: Borrow<str> + ?Sized
    {
        self.map.get_key_value(&KeyStr::new(key.borrow()))
    }

    ///
    /// Finds the index of a key in the map.
    ///
    /// This operation is O(n).
    ///
    pub fn find_key<Q>(
        &self,
        needle: &Q
    ) -> Result<usize, ()>
    where
        Q: Borrow<str> + ?Sized
    {
        for (i, key) in self.order.iter().enumerate() {
            if key == needle.borrow() {
                return Ok(i)
            }
        }

        Err(())
    }

    /// Inserts a new (key, val) into the map. Values are ordered based on their insertion order.
    pub fn insert(
        &mut self,
        key: String,
        val: V
    ) -> Option<V> {
        let key = KeyString::new(key);
        self.order.push(key.clone());
        self.map.insert(key, val)
    }

    /// Gets an iterator for this map.
    pub fn iter(
        &self
    ) -> IniMapIter<'_, V> {
        IniMapIter {
            map: &self,
            index: 0
        }
    }
}

impl<'a, V> Iterator for IniMapIter<'a, V> {
    type Item = (&'a KeyStr<'a>, &'a V);
    fn next(
        &mut self
    ) -> Option<Self::Item> {
        if self.index < self.map.len() {
            let key = self.map.get_key(self.index);
            let val = self.map.get(key).unwrap();
            self.index += 1;
            Some((key, val))
        } else {
            None
        }
    }
}
