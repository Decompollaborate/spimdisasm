/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::{
    borrow::Borrow,
    fmt,
    hash::Hash,
    ops::{BitAnd, BitOr, BitXor, Sub},
};

#[cfg(not(feature = "hash_tables"))]
use alloc::collections::btree_set::{self, BTreeSet};

#[cfg(feature = "hash_tables")]
use std::collections::hash_set::{self, HashSet};

#[derive(Clone, PartialEq, Eq)]
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

    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.inner.is_disjoint(&other.inner)
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        self.inner.is_subset(&other.inner)
    }

    pub fn is_superset(&self, other: &Self) -> bool {
        self.inner.is_superset(&other.inner)
    }
}

impl<T> UnorderedSet<T>
where
    T: Ord + Hash + Eq,
{
    pub fn insert(&mut self, value: T) -> bool {
        self.inner.insert(value)
    }

    pub fn replace(&mut self, value: T) -> Option<T> {
        self.inner.replace(value)
    }

    pub fn take<Q>(&mut self, value: &Q) -> Option<T>
    where
        T: Borrow<Q>,
        Q: ?Sized + Ord + Hash + Eq,
    {
        self.inner.take(value)
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn remove<Q>(&mut self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: ?Sized + Ord + Hash + Eq,
    {
        self.inner.remove(value)
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.inner.retain(f);
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
    pub fn iter(&self) -> btree_set::Iter<'_, T> {
        self.inner.iter()
    }
    #[cfg(feature = "hash_tables")]
    pub fn iter(&self) -> hash_set::Iter<'_, T> {
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

    #[cfg(not(feature = "hash_tables"))]
    pub fn union<'a>(&'a self, other: &'a Self) -> btree_set::Union<'a, T> {
        self.inner.union(&other.inner)
    }
    #[cfg(feature = "hash_tables")]
    pub fn union<'a>(&'a self, other: &'a Self) -> hash_set::Union<'a, T, std::hash::RandomState> {
        self.inner.union(&other.inner)
    }

    #[cfg(not(feature = "hash_tables"))]
    pub fn difference<'a>(&'a self, other: &'a Self) -> btree_set::Difference<'a, T> {
        self.inner.difference(&other.inner)
    }
    #[cfg(feature = "hash_tables")]
    pub fn difference<'a>(
        &'a self,
        other: &'a Self,
    ) -> hash_set::Difference<'a, T, std::hash::RandomState> {
        self.inner.difference(&other.inner)
    }

    #[cfg(not(feature = "hash_tables"))]
    pub fn symmetric_difference<'a>(
        &'a self,
        other: &'a Self,
    ) -> btree_set::SymmetricDifference<'a, T> {
        self.inner.symmetric_difference(&other.inner)
    }
    #[cfg(feature = "hash_tables")]
    pub fn symmetric_difference<'a>(
        &'a self,
        other: &'a Self,
    ) -> hash_set::SymmetricDifference<'a, T, std::hash::RandomState> {
        self.inner.symmetric_difference(&other.inner)
    }
}

impl<T> fmt::Debug for UnorderedSet<T>
where
    T: Ord + Hash + Eq + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Manually implement Debug to hide the `inner` indirection
        write!(f, "UnorderedSet {:?}", self.inner)
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

impl<T> BitAnd for &UnorderedSet<T>
where
    T: Ord + Hash + Eq + Clone,
{
    type Output = UnorderedSet<T>;

    fn bitand(self, rhs: Self) -> Self::Output {
        UnorderedSet {
            inner: self.inner.bitand(&rhs.inner),
        }
    }
}

impl<T> BitOr for &UnorderedSet<T>
where
    T: Ord + Hash + Eq + Clone,
{
    type Output = UnorderedSet<T>;

    fn bitor(self, rhs: Self) -> Self::Output {
        UnorderedSet {
            inner: self.inner.bitor(&rhs.inner),
        }
    }
}

impl<T> BitXor for &UnorderedSet<T>
where
    T: Ord + Hash + Eq + Clone,
{
    type Output = UnorderedSet<T>;

    fn bitxor(self, rhs: Self) -> Self::Output {
        UnorderedSet {
            inner: self.inner.bitxor(&rhs.inner),
        }
    }
}

impl<T> Sub for &UnorderedSet<T>
where
    T: Ord + Hash + Eq + Clone,
{
    type Output = UnorderedSet<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        UnorderedSet {
            inner: self.inner.sub(&rhs.inner),
        }
    }
}
