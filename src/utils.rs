use std::hash::Hash;
use ahash::{AHasher, RandomState, AHashMap};

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Container<T: Hash + Eq + Sized + std::fmt::Display> {
    storage: AHashMap<T, bool>,
}

impl<T: Hash + Eq + Sized + std::fmt::Display> Container<T> {
    pub fn new() -> Self {
        Self {
            storage: AHashMap::new()
        }
    }

    pub fn from(vec: Vec<T>) -> Self {
        let mut container = Self::new();
        for element in vec {
            container.storage.insert(element, false);
        }
        container
    }

    #[inline]
    pub fn contains(&self, element: &T) -> bool {
        self.storage.contains_key(element)
    }

    pub fn push(&mut self, element: T) {
        self.storage.insert(element, false);
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
