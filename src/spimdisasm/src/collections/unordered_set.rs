/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{borrow::Borrow, hash::Hash};

#[cfg(not(feature = "hash_tables"))]
use alloc::collections::btree_set::{self, BTreeSet};

#[cfg(feature = "hash_tables")]
use std::collections::hash_set::{self, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnorderedSet<T>
where
    T: Ord + Hash + Eq,
{
    #[cfg(not(feature = "hash_tables"))]
    inner: BTreeSet<T>,
    #[cfg(feature = "hash_tables")]
    inner: HashSet<T>,
}

impl<T> UnorderedSet<T>
where
    T: Ord + Hash + Eq,
{
    pub fn new() -> Self {
        Self {
            #[cfg(not(feature = "hash_tables"))]
            inner: BTreeSet::new(),
            #[cfg(feature = "hash_tables")]
            inner: HashSet::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        if capacity != 0 {
            // A silly way to avoid unused warnings without prefixing the parameter with an underscore.
        }

        Self {
            #[cfg(not(feature = "hash_tables"))]
            inner: BTreeSet::new(),
            #[cfg(feature = "hash_tables")]
            inner: HashSet::with_capacity(capacity),
        }
    }
}

impl<T> UnorderedSet<T>
where
    T: Ord + Hash + Eq,
{
    pub fn insert(&mut self, value: T) -> bool {
        self.inner.insert(value)
    }

    pub fn get<Q>(&self, value: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: ?Sized + Ord + Hash + Eq,
    {
        self.inner.get(value)
    }

    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: ?Sized + Ord + Hash + Eq,
    {
        self.inner.contains(value)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn append(&mut self, other: &mut Self) {
        #[cfg(not(feature = "hash_tables"))]
        {
            self.inner.append(&mut other.inner);
        }
        #[cfg(feature = "hash_tables")]
        {
            self.inner.extend(other.inner.drain())
        }
    }
}

impl<T> UnorderedSet<T>
where
    T: Ord + Hash + Eq,
{
    #[cfg(not(feature = "hash_tables"))]
    pub fn iter(&self) -> btree_set::Iter<T> {
        self.inner.iter()
    }
    #[cfg(feature = "hash_tables")]
    pub fn iter(&self) -> hash_set::Iter<T> {
        self.inner.iter()
    }

    #[cfg(not(feature = "hash_tables"))]
    pub fn intersection<'a>(&'a self, other: &'a Self) -> btree_set::Intersection<'a, T> {
        self.inner.intersection(&other.inner)
    }
    #[cfg(feature = "hash_tables")]
    pub fn intersection<'a>(
        &'a self,
        other: &'a Self,
    ) -> hash_set::Intersection<'a, T, std::hash::RandomState> {
        self.inner.intersection(&other.inner)
    }
}

impl<T> Default for UnorderedSet<T>
where
    T: Ord + Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> IntoIterator for &'a UnorderedSet<T>
where
    T: Ord + Hash + Eq,
{
    type Item = &'a T;
    #[cfg(not(feature = "hash_tables"))]
    type IntoIter = btree_set::Iter<'a, T>;
    #[cfg(feature = "hash_tables")]
    type IntoIter = hash_set::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> IntoIterator for UnorderedSet<T>
where
    T: Ord + Hash + Eq,
{
    type Item = T;
    #[cfg(not(feature = "hash_tables"))]
    type IntoIter = btree_set::IntoIter<T>;
    #[cfg(feature = "hash_tables")]
    type IntoIter = hash_set::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a, T> Extend<&'a T> for UnorderedSet<T>
where
    T: 'a + Ord + Hash + Eq + Copy,
{
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.inner.extend(iter)
    }
}

impl<T> Extend<T> for UnorderedSet<T>
where
    T: Ord + Hash + Eq,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.inner.extend(iter)
    }
}

impl<T> FromIterator<T> for UnorderedSet<T>
where
    T: Ord + Hash + Eq,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut s = Self::new();
        s.extend(iter);
        s
    }
}
