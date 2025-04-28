/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::collections::btree_map::{self, BTreeMap};
use core::{
    fmt,
    marker::PhantomData,
    ops::{Add, RangeBounds},
};

#[cfg(not(feature = "nightly"))]
use ::polonius_the_crab::prelude::*;

#[cfg(feature = "nightly")]
use core::ops::Bound;

use crate::addresses::Size;

pub type Range<'a, K, V> = btree_map::Range<'a, K, V>;
pub type RangeMut<'a, K, V> = btree_map::RangeMut<'a, K, V>;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AddendedOrderedMap<K, V, SIZE = Size>
where
    K: Ord + Copy + Add<SIZE, Output = K>,
    V: SizedValue<SIZE>,
{
    inner: BTreeMap<K, V>,
    phantom: PhantomData<SIZE>,
}

impl<K, V, SIZE> AddendedOrderedMap<K, V, SIZE>
where
    K: Ord + Copy + Add<SIZE, Output = K>,
    V: SizedValue<SIZE>,
{
    pub const fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
            phantom: PhantomData,
        }
    }
}

impl<K, V, SIZE> AddendedOrderedMap<K, V, SIZE>
where
    K: Ord + Copy + Add<SIZE, Output = K>,
    V: SizedValue<SIZE>,
{
    #[must_use]
    pub fn find(&self, key: &K, settings: FindSettings) -> Option<&V> {
        if !settings.allow_addend {
            self.inner.get(key)
        } else {
            let mut range = self.inner.range(..=key);

            if let Some((other_key, v)) = range.next_back() {
                if other_key == key || *key < *other_key + v.size() {
                    Some(v)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    /*
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: ?Sized + Ord,
    {
        self.inner.contains_key(key)
    }
    */

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<K, V, SIZE> AddendedOrderedMap<K, V, SIZE>
where
    K: Ord + Copy + Add<SIZE, Output = K>,
    V: SizedValue<SIZE>,
{
    #[must_use]
    pub fn find_mut(&mut self, key: &K, settings: FindSettings) -> Option<&mut V> {
        if !settings.allow_addend {
            self.inner.get_mut(key)
        } else {
            let mut range = self.inner.range_mut(..=key);

            if let Some((other_key, v)) = range.next_back() {
                if other_key == key || *key < *other_key + v.size() {
                    Some(v)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}

#[cfg(not(feature = "nightly"))]
fn add_impl<'slf, K, V, SIZE, F>(
    mut slf: &'slf mut AddendedOrderedMap<K, V, SIZE>,
    key: &K,
    settings: FindSettings,
    default: F,
) -> (&'slf mut V, bool)
where
    K: Ord + Copy + Add<SIZE, Output = K>,
    V: SizedValue<SIZE>,
    F: FnOnce() -> (K, V),
{
    // TODO: get rid of the polonius stuff when the new borrow checker has been released.

    polonius!(|slf| -> (&'polonius mut V, bool) {
        if let Some(x) = slf.find_mut(key, settings) {
            polonius_return!((x, false));
        }
    });

    let (k, v) = default();
    let entry = slf.inner.entry(k);

    let newly_created = matches!(entry, btree_map::Entry::Vacant(_));
    (entry.or_insert(v), newly_created)
}

#[cfg(feature = "nightly")]
fn add_impl<'slf, K, V, SIZE, F>(
    slf: &'slf mut AddendedOrderedMap<K, V, SIZE>,
    key: &K,
    settings: FindSettings,
    default: F,
) -> (&'slf mut V, bool)
where
    K: Ord + Copy + Add<SIZE, Output = K>,
    V: SizedValue<SIZE>,
    F: FnOnce() -> (K, V),
{
    let mut cursor = slf.inner.upper_bound_mut(Bound::Included(key));

    let must_insert_new = if let Some((other_key, v)) = cursor.peek_prev() {
        if key == other_key {
            false
        } else if !settings.allow_addend {
            true
        } else {
            *key >= *other_key + v.size()
        }
    } else {
        true
    };

    if must_insert_new {
        let (k, v) = default();
        cursor
            .insert_before(k, v)
            .expect("This should not be able to panic");
    }

    //let sym = unsafe { &mut *(cursor.peek_prev().unwrap().1 as *mut SymbolMetadata) };
    (into_prev_and_next(cursor).0.unwrap().1, must_insert_new)
}

#[cfg(feature = "nightly")]
fn into_prev_and_next<'a, K, V>(
    mut cursor: btree_map::CursorMut<'a, K, V>,
) -> (Option<(&'a K, &'a mut V)>, Option<(&'a K, &'a mut V)>) {
    let prev: Option<(&'a K, &'a mut V)> = cursor.peek_prev().map(|(k, v)| {
        let ptr_k = k as *const K;
        let ptr_v = v as *mut V;
        unsafe { (&*ptr_k, &mut *ptr_v) }
    });
    let next: Option<(&'a K, &'a mut V)> = cursor.peek_next().map(|(k, v)| {
        let ptr_k = k as *const K;
        let ptr_v = v as *mut V;
        unsafe { (&*ptr_k, &mut *ptr_v) }
    });

    (prev, next)
}

impl<K, V, SIZE> AddendedOrderedMap<K, V, SIZE>
where
    K: Ord + Copy + Add<SIZE, Output = K>,
    V: SizedValue<SIZE>,
{
    pub fn find_mut_or_insert_with<F>(
        &mut self,
        key: K,
        settings: FindSettings,
        default: F,
    ) -> (&mut V, bool)
    where
        K: Copy,
        F: FnOnce() -> V,
    {
        add_impl(self, &key, settings, || (key, default()))
    }

    pub fn find_mut_or_insert_with_key_value<F>(
        &mut self,
        key: &K,
        settings: FindSettings,
        default: F,
    ) -> (&mut V, bool)
    where
        F: FnOnce() -> (K, V),
    {
        add_impl(self, key, settings, default)
    }
}

impl<K, V, SIZE> AddendedOrderedMap<K, V, SIZE>
where
    K: Ord + Copy + Add<SIZE, Output = K>,
    V: SizedValue<SIZE>,
{
    /*
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn remove<Q>(&mut self, value: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Ord,
    {
        self.inner.remove(value)
    }

    pub fn remove_entry<Q>(&mut self, value: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: ?Sized + Ord,
    {
        self.inner.remove_entry(value)
    }
    */

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        self.inner.retain(f);
    }
}

impl<K, V, SIZE> AddendedOrderedMap<K, V, SIZE>
where
    K: Ord + Copy + Add<SIZE, Output = K>,
    V: SizedValue<SIZE>,
{
    pub fn iter(&self) -> btree_map::Iter<K, V> {
        self.inner.iter()
    }

    /*
    pub fn iter_mut(&mut self) -> btree_map::IterMut<K, V> {
        self.inner.iter_mut()
    }
    */

    pub fn range<R>(&self, range: R) -> Range<'_, K, V>
    where
        R: RangeBounds<K>,
    {
        self.inner.range(range)
    }

    pub fn range_mut<R>(&mut self, range: R) -> RangeMut<'_, K, V>
    where
        R: RangeBounds<K>,
    {
        self.inner.range_mut(range)
    }

    pub fn keys(&self) -> btree_map::Keys<K, V> {
        self.inner.keys()
    }

    pub fn values(&self) -> btree_map::Values<K, V> {
        self.inner.values()
    }

    pub fn values_mut(&mut self) -> btree_map::ValuesMut<K, V> {
        self.inner.values_mut()
    }

    pub fn into_keys(self) -> btree_map::IntoKeys<K, V> {
        self.inner.into_keys()
    }

    pub fn into_values(self) -> btree_map::IntoValues<K, V> {
        self.inner.into_values()
    }
}

impl<K, V, SIZE> fmt::Debug for AddendedOrderedMap<K, V, SIZE>
where
    K: Ord + Copy + Add<SIZE, Output = K> + fmt::Debug,
    V: SizedValue<SIZE> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Manually implement Debug to hide the `inner` indirection
        write!(f, "AddendedOrderedMap {:?}", self.inner)
    }
}

impl<K, V, SIZE> Default for AddendedOrderedMap<K, V, SIZE>
where
    K: Ord + Copy + Add<SIZE, Output = K>,
    V: SizedValue<SIZE>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, K, V, SIZE> IntoIterator for &'a AddendedOrderedMap<K, V, SIZE>
where
    K: Ord + Copy + Add<SIZE, Output = K>,
    V: SizedValue<SIZE>,
{
    type Item = (&'a K, &'a V);
    type IntoIter = btree_map::Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/*
impl<'a, K, V> IntoIterator for &'a mut AddendedOrderedMap<K, V>
where
    K: Ord + Copy + Add<SIZE, Output = K>,
    V: SizedValue<SIZE>,
{
    type Item = (&'a K, &'a mut V);
    type IntoIter = btree_map::IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
*/

impl<K, V, SIZE> IntoIterator for AddendedOrderedMap<K, V, SIZE>
where
    K: Ord + Copy + Add<SIZE, Output = K>,
    V: SizedValue<SIZE>,
{
    type Item = (K, V);
    type IntoIter = btree_map::IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

/*
impl<'a, K, V> Extend<(&'a K, &'a V)> for AddendedOrderedMap<K, V>
where
    K: 'a + Ord,
    V: SizedValue<SIZE>,
{
    fn extend<I: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: I) {
        self.inner.extend(iter)
    }
}
*/

/*
impl<K, V> Extend<(K, V)> for AddendedOrderedMap<K, V>
where
    K: Ord + Copy + Add<SIZE, Output = K>,
    V: SizedValue<SIZE>,
{
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        self.inner.extend(iter)
    }
}
*/

/*
impl<K, V> FromIterator<(K, V)> for AddendedOrderedMap<K, V>
where
    K: Ord + Copy + Add<SIZE, Output = K>,
    V: SizedValue<SIZE>,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut s = Self::new();
        s.extend(iter);
        s
    }
}
*/

pub trait SizedValue<SIZE = Size> {
    fn size(&self) -> SIZE;
}

// TODO: use the bitflags crate instead of a plain struct
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FindSettings {
    allow_addend: bool,
}

impl FindSettings {
    pub const fn new(allow_addend: bool) -> Self {
        Self { allow_addend }
    }
}
