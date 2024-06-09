use ahash::{AHashMap, AHasher, HashSet, HashSetExt, RandomState};
use color_eyre::eyre::{bail, eyre, Result};
use std::collections::hash_set::Difference;
use std::fmt::Display;
use std::hash::Hash;
use std::thread;
use std::thread::JoinHandle;
use trait_set::trait_set;

#[cfg(not(feature = "use_kanal"))]
use crossbeam_channel::{unbounded, bounded, Receiver, Sender};
#[cfg(feature = "use_kanal")]
use kanal::{unbounded, bounded, Receiver, Sender};

pub fn create_channel<T>(size: i32) -> (Sender<T>, Receiver<T>) {
    if size == -1 {
        unbounded()
    } else {
        bounded(size as usize)
    }
}

pub type TerminationSender = Sender<()>;
pub type TerminationReceiver = Receiver<()>;

#[derive(Clone, Debug)]
pub struct TerminationStatus {
    sender: TerminationSender,
    receiver: TerminationReceiver,
}

impl TerminationStatus {
    pub fn default() -> Self {
        Self::new(-1)
    }

    pub fn new(size: i32) -> Self {
        let (sender, receiver) = create_channel(size);
        Self {
            sender,
            receiver,
        }
    }

    pub fn check(&self) -> bool {
        self.receiver.try_recv().is_ok()
    }

    fn _notify_all(&self) -> Result<()> {
        self.sender.send(())?;
        Ok(())
    }

    fn _notify_and_panic(&self, err: color_eyre::eyre::Report) {
        self._notify_all().unwrap();
        panic!("{}", err);
    }

    // pub fn notify_all_pass_error(&self, thread_handle: ThreadHandle){
    //     match thread_handle.join(){
    //         Ok(_) => {}
    //         Err(err) => {
    //             self._notify_and_panic(crate::err_eyre!(err));
    //         }
    //     }
    // }

    pub fn check_result(&self, result: Result<()>) {
        match result {
            Ok(_) => {}
            Err(err) => {
                self._notify_and_panic(err);
            }
        }
    }

    //methods below require closures, capturing, and cloning

    //notify_all_pass_error
    // pub fn run_with_check<F: FnOnce() -> Result<()> + Send + 'static>(&self, f: F) {
    //     self.check_result(f());
    // }
    //
    // pub fn spawn_with_check<F: FnOnce() -> Result<()> + Send + 'static>(&self, f: F) -> ThreadHandle {
    //     let self_copy = self.clone();
    //     thread::spawn(move || {
    //         Self::run_with_check(&self_copy, f);
    //     })
    // }
}

pub type ThreadHandle = JoinHandle<()>;
pub type ThreadHandleOption<'a> = Option<&'a ThreadHandle>;

pub fn check_thread_handle(thread_handle: ThreadHandleOption) -> Result<()> {
    if let Some(thread_handle) = thread_handle {
        if thread_handle.is_finished() {
            bail!("Thread panicked")
        }
    }
    Ok(())
}

pub fn try_unwrap_thread(thread_handle: ThreadHandle) {
    if thread_handle.is_finished() {
        thread_handle.join().unwrap();
    };
}

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
) -> Result<&'a V> {
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

#[inline]
pub fn option_to_string<T: Display>(value: Option<T>) -> String {
    match value {
        None => "None".to_string(),
        Some(value) => value.to_string(),
    }
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
