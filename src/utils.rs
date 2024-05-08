use ahash::{AHashMap, AHasher, HashSet, HashSetExt, RandomState};
use std::collections::hash_set::Difference;
use std::fmt::Display;
use std::hash::Hash;
use trait_set::trait_set;

trait_set! {
    pub trait ContainerElement = Copy + Eq + Sized + Display;
    pub trait SetElement = Hash + Eq;
}

pub type SetDiff<'a, T> = Difference<'a, T, RandomState>;

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Container<T: ContainerElement> {
    storage: Vec<T>,
}

impl<T: ContainerElement> Container<T> {
    pub fn new() -> Self {
        Self { storage: vec![] }
    }

    pub fn from(vec: Vec<T>) -> Self {
        let mut container = Self::new();
        for element in vec {
            container.push(element);
        }
        container
    }

    #[inline]
    pub fn contains(&self, element: &T) -> bool {
        let len = self.storage.len();
        for ind in 0..len {
            if self.storage[ind] == *element {
                return true;
            }
        }
        false
    }

    #[inline]
    pub fn difference(&self, other: &Container<T>) -> Vec<T> {
        let mut diff = vec![];

        let len = self.storage.len();
        for ind in 0..len {
            let element = self.storage[ind];
            if !other.contains(&element) {
                diff.push(element)
            }
        }
        diff
    }

    pub fn push(&mut self, element: T) {
        self.storage.push(element);
    }
}

#[inline]
pub fn get_or_default<'a, K: Hash + Eq + Sized + std::fmt::Display, V: Default + Copy>(
    m: &'a AHashMap<K, V>,
    key: &'a K,
) -> V {
    match m.get(key) {
        None => V::default(),
        Some(value) => *value,
    }
}

#[inline]
pub fn get_or_err<'a, K: Hash + Eq + Sized + std::fmt::Display, V>(
    m: &'a AHashMap<K, V>,
    key: &'a K,
) -> color_eyre::Result<&'a V> {
    m.get(key)
        .ok_or_else(|| color_eyre::eyre::Report::msg(format!("No mapping for '{}'", key)))
}

#[inline(always)]
pub fn are_options_equal<T: PartialEq>(value1: Option<T>, value2: Option<T>) -> bool {
    match (value1, value2) {
        (Some(value1), Some(value2)) => value1 == value2,
        (None, None) => true,
        _ => false,
    }
}

#[inline(always)]
pub fn are_options_different<T: PartialEq>(value1: Option<T>, value2: Option<T>) -> bool {
    !are_options_equal(value1, value2)
}

#[macro_export]
macro_rules! err_eyre {
    ($err:expr $(,)?) => {{
        color_eyre::eyre::eyre!($err.to_string())
    }};
}

#[macro_export]
macro_rules! exec_or_eyre {
    ($f: expr) => {{
        $f.map_err(|error| $crate::err_eyre!(error))
    }};
}
