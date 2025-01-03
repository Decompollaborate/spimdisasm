/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::collections::btree_map::{self, BTreeMap};
use core::ops::{Add, RangeBounds};

#[cfg(not(feature = "nightly"))]
use ::polonius_the_crab::prelude::*;

#[cfg(feature = "nightly")]
use core::ops::Bound;

use crate::addresses::{Size, SizedAddress};

// TODO: default
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AddendedOrderedMap<K, V>
where
    K: Ord,
    V: SizedAddress,
{
    inner: BTreeMap<K, V>,
}

impl<K, V> AddendedOrderedMap<K, V>
where
    K: Ord,
    V: SizedAddress,
{
    pub const fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }
}

impl<K, V> AddendedOrderedMap<K, V>
where
    K: Ord + Copy + Add<Size, Output = K>,
    V: SizedAddress,
{
    #[must_use]
    pub fn find(&self, key: &K, settings: FindSettings) -> Option<&V> {
        if !settings.allow_addend {
            self.inner.get(key)
        } else {
            let mut range = self.inner.range(..=key);

            if let Some((other_key, v)) = range.next_back() {
                if other_key == key {
                    Some(v)
                } else if settings.check_upper_limit {
                    v.size().and_then(|siz| {
                        if *key < *other_key + siz {
                            Some(v)
                        } else {
                            None
                        }
                    })
                } else {
                    Some(v)
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

impl<K, V> AddendedOrderedMap<K, V>
where
    K: Ord + Copy + Add<Size, Output = K>,
    V: SizedAddress,
{
    #[must_use]
    pub fn find_mut(&mut self, key: &K, settings: FindSettings) -> Option<&mut V> {
        if !settings.allow_addend {
            self.inner.get_mut(key)
        } else {
            let mut range = self.inner.range_mut(..=key);

            if let Some((other_key, v)) = range.next_back() {
                if other_key == key {
                    Some(v)
                } else if settings.check_upper_limit {
                    v.size().and_then(|siz| {
                        if *key < *other_key + siz {
                            Some(v)
                        } else {
                            None
                        }
                    })
                } else {
                    Some(v)
                }
            } else {
                None
            }
        }
    }
}

#[cfg(not(feature = "nightly"))]
fn add_impl<'slf, K, V, F>(
    mut slf: &'slf mut AddendedOrderedMap<K, V>,
    key: &K,
    settings: FindSettings,
    default: F,
) -> &'slf mut V
where
    K: Ord + Copy + Add<Size, Output = K>,
    V: SizedAddress,
    F: FnOnce() -> V,
{
    // TODO: get rid of the polonius stuff when the new borrow checker has been released.

    polonius!(|slf| -> &'polonius mut V {
        if let Some(x) = slf.find_mut(key, settings) {
            polonius_return!(x);
        }
    });

    let entry = slf.inner.entry(*key);

    entry.or_insert_with(default)
}

#[cfg(feature = "nightly")]
fn add_impl<'slf, K, V, F>(
    slf: &'slf mut AddendedOrderedMap<K, V>,
    key: &K,
    settings: FindSettings,
    default: F,
) -> &'slf mut V
where
    K: Ord + Copy + Add<Size, Output = K>,
    V: SizedAddress,
    F: FnOnce() -> V,
{
    let mut cursor = slf.inner.upper_bound_mut(Bound::Included(key));

    let must_insert_new = if let Some((other_key, v)) = cursor.peek_prev() {
        if key == other_key {
            false
        } else if !settings.allow_addend {
            true
        } else if !settings.check_upper_limit {
            false
        } else if let Some(siz) = v.size() {
            *key >= *other_key + siz
        } else {
            true
        }
    } else {
        true
    };

    if must_insert_new {
        cursor
            .insert_before(*key, default())
            .expect("This should not be able to panic");
    }

    //let sym = unsafe { &mut *(cursor.peek_prev().unwrap().1 as *mut SymbolMetadata) };
    into_prev_and_next(cursor).0.unwrap().1
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

impl<K, V> AddendedOrderedMap<K, V>
where
    K: Ord + Copy + Add<Size, Output = K>,
    V: SizedAddress,
{
    pub fn find_mut_or_insert_with<F>(
        &mut self,
        key: &K,
        settings: FindSettings,
        default: F,
    ) -> &mut V
    where
        F: FnOnce() -> V,
    {
        add_impl(self, key, settings, default)
    }
}

/*
impl<K, V> AddendedOrderedMap<K, V>
where
    K: Ord,
    V: SizedAddress,
{
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

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        self.inner.retain(f);
    }
}
*/

impl<K, V> AddendedOrderedMap<K, V>
where
    K: Ord,
    V: SizedAddress,
{
    pub fn iter(&self) -> btree_map::Iter<K, V> {
        self.inner.iter()
    }

    /*
    pub fn iter_mut(&mut self) -> btree_map::IterMut<K, V> {
        self.inner.iter_mut()
    }
    */

    pub fn range<R>(&self, range: R) -> btree_map::Range<'_, K, V>
    where
        R: RangeBounds<K>,
    {
        self.inner.range(range)
    }

    /*
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
    */
}

impl<K, V> Default for AddendedOrderedMap<K, V>
where
    K: Ord,
    V: SizedAddress,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, K, V> IntoIterator for &'a AddendedOrderedMap<K, V>
where
    K: Ord,
    V: SizedAddress,
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
    K: Ord,
    V: SizedAddress,
{
    type Item = (&'a K, &'a mut V);
    type IntoIter = btree_map::IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
*/

impl<K, V> IntoIterator for AddendedOrderedMap<K, V>
where
    K: Ord,
    V: SizedAddress,
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
    K: 'a + Ord + Copy,
    V: SizedAddress + Copy,
{
    fn extend<I: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: I) {
        self.inner.extend(iter)
    }
}
*/

/*
impl<K, V> Extend<(K, V)> for AddendedOrderedMap<K, V>
where
    K: Ord,
    V: SizedAddress,
{
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        self.inner.extend(iter)
    }
}
*/

/*
impl<K, V> FromIterator<(K, V)> for AddendedOrderedMap<K, V>
where
    K: Ord,
    V: SizedAddress,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut s = Self::new();
        s.extend(iter);
        s
    }
}
*/

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FindSettings {
    allow_addend: bool,
    check_upper_limit: bool,
}

impl FindSettings {
    pub const fn default() -> Self {
        Self {
            allow_addend: true,
            check_upper_limit: true,
        }
    }

    pub const fn new() -> Self {
        Self::default()
    }

    pub const fn with_allow_addend(self, allow_addend: bool) -> Self {
        Self {
            allow_addend,
            ..self
        }
    }

    pub const fn with_check_upper_limit(self, check_upper_limit: bool) -> Self {
        Self {
            check_upper_limit,
            ..self
        }
    }
}
impl Default for FindSettings {
    fn default() -> Self {
        Self::default()
    }
}
