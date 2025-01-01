/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{borrow::Borrow, hash::Hash};

#[cfg(not(feature = "hash_tables"))]
use alloc::collections::btree_map::{self, BTreeMap};

#[cfg(feature = "hash_tables")]
use std::collections::hash_map::{self, HashMap};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnorderedMap<K, V>
where
    K: Ord + Hash + Eq,
{
    #[cfg(not(feature = "hash_tables"))]
    inner: BTreeMap<K, V>,
    #[cfg(feature = "hash_tables")]
    inner: HashMap<K, V>,
}

impl<K, V> UnorderedMap<K, V>
where
    K: Ord + Hash + Eq,
{
    pub fn new() -> Self {
        Self {
            #[cfg(not(feature = "hash_tables"))]
            inner: BTreeMap::new(),
            #[cfg(feature = "hash_tables")]
            inner: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        if capacity != 0 {
            // A silly way to avoid unused warnings without prefixing the parameter with an underscore.
        }

        Self {
            #[cfg(not(feature = "hash_tables"))]
            inner: BTreeMap::new(),
            #[cfg(feature = "hash_tables")]
            inner: HashMap::with_capacity(capacity),
        }
    }
}

impl<K, V> UnorderedMap<K, V>
where
    K: Ord + Hash + Eq,
{
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.inner.insert(key, value)
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Ord + Hash + Eq,
    {
        self.inner.get(key)
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Ord + Hash + Eq,
    {
        self.inner.get_mut(key)
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: ?Sized + Ord + Hash + Eq,
    {
        self.inner.contains_key(key)
    }

    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        Entry {
            inner: self.inner.entry(key),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<K, V> UnorderedMap<K, V>
where
    K: Ord + Hash + Eq,
{
    #[cfg(not(feature = "hash_tables"))]
    pub fn iter(&self) -> btree_map::Iter<K, V> {
        self.inner.iter()
    }
    #[cfg(feature = "hash_tables")]
    pub fn iter(&self) -> hash_map::Iter<K, V> {
        self.inner.iter()
    }

    #[cfg(not(feature = "hash_tables"))]
    pub fn iter_mut(&mut self) -> btree_map::IterMut<K, V> {
        self.inner.iter_mut()
    }
    #[cfg(feature = "hash_tables")]
    pub fn iter_mut(&mut self) -> hash_map::IterMut<K, V> {
        self.inner.iter_mut()
    }

    #[cfg(not(feature = "hash_tables"))]
    pub fn keys(&self) -> btree_map::Keys<K, V> {
        self.inner.keys()
    }
    #[cfg(feature = "hash_tables")]
    pub fn keys(&self) -> hash_map::Keys<K, V> {
        self.inner.keys()
    }

    #[cfg(not(feature = "hash_tables"))]
    pub fn values(&self) -> btree_map::Values<K, V> {
        self.inner.values()
    }
    #[cfg(feature = "hash_tables")]
    pub fn values(&self) -> hash_map::Values<K, V> {
        self.inner.values()
    }
}

impl<K, V> Default for UnorderedMap<K, V>
where
    K: Ord + Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, K, V> IntoIterator for &'a UnorderedMap<K, V>
where
    K: Ord + Hash + Eq,
{
    type Item = (&'a K, &'a V);
    #[cfg(not(feature = "hash_tables"))]
    type IntoIter = btree_map::Iter<'a, K, V>;
    #[cfg(feature = "hash_tables")]
    type IntoIter = hash_map::Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut UnorderedMap<K, V>
where
    K: Ord + Hash + Eq,
{
    type Item = (&'a K, &'a mut V);
    #[cfg(not(feature = "hash_tables"))]
    type IntoIter = btree_map::IterMut<'a, K, V>;
    #[cfg(feature = "hash_tables")]
    type IntoIter = hash_map::IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K, V> IntoIterator for UnorderedMap<K, V>
where
    K: Ord + Hash + Eq,
{
    type Item = (K, V);
    #[cfg(not(feature = "hash_tables"))]
    type IntoIter = btree_map::IntoIter<K, V>;
    #[cfg(feature = "hash_tables")]
    type IntoIter = hash_map::IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a, K, V> Extend<(&'a K, &'a V)> for UnorderedMap<K, V>
where
    K: 'a + Ord + Hash + Eq + Copy,
    V: Copy,
{
    fn extend<I: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: I) {
        self.inner.extend(iter)
    }
}

impl<K, V> Extend<(K, V)> for UnorderedMap<K, V>
where
    K: Ord + Hash + Eq,
{
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        self.inner.extend(iter)
    }
}

impl<K, V> FromIterator<(K, V)> for UnorderedMap<K, V>
where
    K: Ord + Hash + Eq,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut s = Self::new();
        s.extend(iter);
        s
    }
}

#[derive(Debug)]
pub struct Entry<'a, K: 'a, V: 'a>
where
    K: Ord,
{
    #[cfg(not(feature = "hash_tables"))]
    inner: btree_map::Entry<'a, K, V>,
    #[cfg(feature = "hash_tables")]
    inner: hash_map::Entry<'a, K, V>,
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: Ord,
{
    pub fn or_insert(self, default: V) -> &'a mut V {
        self.inner.or_insert(default)
    }

    pub fn or_insert_with<F>(self, default: F) -> &'a mut V
    where
        F: FnOnce() -> V,
    {
        self.inner.or_insert_with(default)
    }

    pub fn or_insert_with_key<F>(self, default: F) -> &'a mut V
    where
        F: FnOnce(&K) -> V,
    {
        self.inner.or_insert_with_key(default)
    }

    pub fn key(&self) -> &K {
        self.inner.key()
    }

    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        Self {
            inner: self.inner.and_modify(f),
        }
    }
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: Ord,
    V: Default,
{
    pub fn or_default(self) -> &'a mut V {
        self.inner.or_default()
    }
}