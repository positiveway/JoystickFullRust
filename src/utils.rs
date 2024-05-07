use ahash::{AHashMap, AHasher, HashSet, HashSetExt, RandomState};
use std::collections::hash_set::Difference;
use std::fmt::Display;
use std::hash::Hash;
use trait_set::trait_set;

trait_set! {
    pub trait ContainerElement = PartialOrd + Hash + Eq + Sized + Display;
    pub trait SetElement = Hash + Eq;
}

pub type SetDiff<'a, T> = Difference<'a, T, RandomState>;


#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Container<T: ContainerElement> {
    storage: HashSet<T>,
}

impl<T: ContainerElement> Container<T> {
    pub fn new() -> Self {
        Self {
            storage: HashSet::new(),
        }
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
        self.storage.contains(element)
    }

    pub fn push(&mut self, element: T) {
        self.storage.insert(element);
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
