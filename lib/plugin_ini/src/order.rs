//!
//! @file order.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Map implementation which allows iteration in a strict order.
//! @bug No known bugs.
//!

use std::rc::Rc;
use std::vec::Vec;
use std::collections::HashMap;
use std::hash::Hash;
use std::borrow::Borrow;

/// A hash-map which maintains a strict ordering on the keys it contains.
pub struct OrderMap<K, V> {
    order: Vec<Rc<K>>,
    map: HashMap<Rc<K>, V>,
}

/// Iterates over the elements in an ordered map, in order.
pub struct OrderMapIter<'a, K, V> {
    map: &'a OrderMap<K, V>,
    index: usize
}

impl<K, V> OrderMap<K, V> {
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
        K: Hash + Eq,
        Q: AsRef<K> + ?Sized
    {
        self.map.get(key.as_ref())
    }

    /// Gets a mutable reference to the element with the given key.
    pub fn get_mut<'a, Q>(
        &'a mut self,
        key: &Q
    ) -> Option<&'a mut V>
    where
        K: Hash + Eq,
        Q: AsRef<K> + ?Sized
    {
        self.map.get_mut(key.as_ref())
    }

    /// Gets the key at the ith position in the map.
    pub fn get_key<'a, Q>(
        &'a self,
        i: usize
    ) -> &'a Q
    where
        K: Borrow<Q>,
        Q: Hash + Eq
    {
        <K as Borrow<Q>>::borrow(&self.order[i])
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
        K: Borrow<Q>,
        Q: Eq
    {
        for (i, key) in self.order.iter().enumerate() {
            if <K as Borrow<Q>>::borrow(&key) == needle {
                return Ok(i)
            }
        }

        Err(())
    }

    /// Inserts a new (key, val) into the map.
    pub fn insert(
        &mut self,
        i: usize,
        key: K,
        val: V
    ) -> Option<V>
    where
        K: Hash + Eq
    {
        let key = Rc::new(key);
        self.order.insert(i, key.clone());
        self.map.insert(key, val)
    }

    /// Gets an iterator for this map.
    pub fn iter(
        &self
    ) -> OrderMapIter<'_, K, V> {
        OrderMapIter {
            map: &self,
            index: 0
        }
    }
}

impl<'a, K: Hash + Eq + AsRef<K>, V> Iterator for OrderMapIter<'a, K, V> {
    type Item = (&'a K, &'a V);
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
