use std::collections::HashMap;
use std::hash::Hash;

struct Wrapper<T>(T);

trait Overflow {}
impl<T> Overflow for Wrapper<Wrapper<T>> where Wrapper<T>: Overflow {}
impl Overflow for Wrapper<u32> {}

// Checking whether these two implementations overlap
// tries to prove that either `Wrapper<_>: Overflow` or
// `Wrapper<_>: Copy` do not hold.
//
// The existing solver first checks `Wrapper<_>: Overflow`,
// resulting in overflow and aborting compilation.
//
// The new solver does not abort compilation on overflow and
// considers the implementations to be disjoint, given that
// `Wrapper<_>: Copy` does not hold.
trait MayOverlap {}
impl<T: Overflow + Copy> MayOverlap for T {}
impl<T> MayOverlap for Wrapper<T> {}

fn get_default<'r, K: Hash + Eq + Copy, V: Default>(
    map: &'r mut HashMap<K, V>,
    key: K,
) -> &'r mut V {
    match map.get_mut(&key) { // -------------+ 'r
        Some(value) => value,              // |
        None => {                          // |
            map.insert(key, V::default()); // |
            //  ^~~~~~ ERROR               // |
            map.get_mut(&key).unwrap()     // |
        }                                  // |
    }                                      // |
}

fn main() {
    println!("Hello, world!");
}
