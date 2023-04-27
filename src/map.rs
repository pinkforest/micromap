// Copyright (c) 2023 Yegor Bugayenko
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included
// in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NON-INFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use crate::Map;
use std::borrow::Borrow;

impl<K: PartialEq + Clone, V: Clone, const N: usize> Map<K, V, N> {
    /// Get its total capacity.
    #[inline]
    #[must_use]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Is it empty?
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return the total number of pairs inside.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        let mut busy = 0;
        for i in 0..self.next {
            let p = unsafe { self.pairs[i].assume_init_ref() };
            if p.is_some() {
                busy += 1;
            }
        }
        busy
    }

    /// Does the map contain this key?
    #[inline]
    #[must_use]
    pub fn contains_key(&self, k: &K) -> bool {
        for i in 0..self.next {
            let p = unsafe { self.pairs[i].assume_init_ref() };
            if let Some((bk, _bv)) = &p {
                if bk == k {
                    return true;
                }
            }
        }
        false
    }

    /// Remove by key.
    #[inline]
    pub fn remove(&mut self, k: &K) {
        for i in 0..self.next {
            let p = unsafe { self.pairs[i].assume_init_ref() };
            if let Some(p) = &p {
                if p.0.borrow() == k {
                    self.pairs[i].write(None);
                    break;
                }
            }
        }
    }

    /// Insert a single pair into the map.
    ///
    /// # Panics
    ///
    /// It may panic if there are too many pairs in the map already. Pay attention,
    /// it panics only in "debug" mode. In "release" mode you are going to get
    /// undefined behavior. This is done for the sake of performance, in order to
    /// avoid a repetitive check for the boundary condition on every insert.
    #[inline]
    pub fn insert(&mut self, k: K, v: V) {
        let mut target = self.next;
        let mut i = 0;
        loop {
            if i == self.next {
                debug_assert!(i < N, "No more keys available in the map");
                self.next += 1;
                break;
            }
            let p = unsafe { self.pairs[i].assume_init_ref() };
            if let Some(p) = &p {
                if *p.0.borrow() == k {
                    target = i;
                    break;
                }
            }
            if p.is_none() {
                target = i;
            }
            i += 1;
        }
        self.pairs[target].write(Some((k, v)));
    }

    /// Get a reference to a single value.
    #[inline]
    #[must_use]
    pub fn get<Q: PartialEq + ?Sized>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        for i in 0..self.next {
            let p = unsafe { self.pairs[i].assume_init_ref() };
            if let Some(p) = &p {
                if p.0.borrow() == k {
                    return Some(&p.1);
                }
            }
        }
        None
    }

    /// Get a mutable reference to a single value.
    ///
    /// # Panics
    ///
    /// If can't turn it into a mutable state.
    #[inline]
    #[must_use]
    pub fn get_mut<Q: PartialEq + ?Sized>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
    {
        for i in 0..self.next {
            let p = unsafe { self.pairs[i].assume_init_ref() };
            if let Some(p1) = &p {
                if p1.0.borrow() == k {
                    let p2 = unsafe { self.pairs[i].assume_init_mut() };
                    return Some(&mut p2.as_mut().unwrap().1);
                }
            }
        }
        None
    }

    /// Remove all pairs from it, but keep the space intact for future use.
    #[inline]
    pub fn clear(&mut self) {
        self.next = 0;
    }

    /// Retains only the elements specified by the predicate.
    #[inline]
    pub fn retain<F: Fn(&K, &V) -> bool>(&mut self, f: F) {
        for i in 0..self.next {
            let p = unsafe { self.pairs[i].assume_init_ref() };
            if let Some((k, v)) = &p {
                if !f(k, v) {
                    self.pairs[i].write(None);
                }
            }
        }
    }
}

#[test]
fn insert_and_check_length() {
    let mut m: Map<&str, i32, 10> = Map::new();
    m.insert("first", 42);
    assert_eq!(1, m.len());
    m.insert("second", 16);
    assert_eq!(2, m.len());
    m.insert("first", 16);
    assert_eq!(2, m.len());
}

#[test]
fn overwrites_keys() {
    let mut m: Map<i32, i32, 1> = Map::new();
    m.insert(1, 42);
    m.insert(1, 42);
    assert_eq!(1, m.len());
}

#[test]
#[should_panic]
#[cfg(debug_assertions)]
fn cant_write_into_empty_map() {
    let mut m: Map<i32, i32, 0> = Map::new();
    m.insert(1, 42);
}

#[test]
fn empty_length() {
    let m: Map<u32, u32, 10> = Map::new();
    assert_eq!(0, m.len());
}

#[test]
fn is_empty_check() {
    let mut m: Map<u32, u32, 10> = Map::new();
    assert!(m.is_empty());
    m.insert(42, 42);
    assert!(!m.is_empty());
}

#[test]
fn insert_and_gets() {
    let mut m: Map<&str, i32, 10> = Map::new();
    m.insert("one", 42);
    m.insert("two", 16);
    assert_eq!(16, *m.get(&"two").unwrap());
}

#[test]
fn insert_and_gets_mut() {
    let mut m: Map<i32, [i32; 3], 10> = Map::new();
    m.insert(42, [1, 2, 3]);
    let a = m.get_mut(&42).unwrap();
    a[0] = 500;
    assert_eq!(500, m.get(&42).unwrap()[0]);
}

#[test]
fn checks_key() {
    let mut m: Map<&str, i32, 10> = Map::new();
    m.insert("one", 42);
    assert!(m.contains_key(&"one"));
    assert!(!m.contains_key(&"another"));
}

#[test]
fn gets_missing_key() {
    let mut m: Map<&str, i32, 10> = Map::new();
    m.insert("one", 42);
    assert!(m.get(&"two").is_none());
}

#[test]
fn mut_gets_missing_key() {
    let mut m: Map<&str, i32, 10> = Map::new();
    m.insert("one", 42);
    assert!(m.get_mut(&"two").is_none());
}

#[test]
fn removes_simple_pair() {
    let mut m: Map<&str, i32, 10> = Map::new();
    m.insert("one", 42);
    m.remove(&"one");
    m.remove(&"another");
    assert!(m.get(&"one").is_none());
}

#[cfg(test)]
#[derive(Clone)]
struct Foo {
    v: [u32; 3],
}

#[test]
fn insert_struct() {
    let mut m: Map<u8, Foo, 8> = Map::new();
    let foo = Foo { v: [1, 2, 100] };
    m.insert(1, foo);
    assert_eq!(100, m.into_iter().next().unwrap().1.v[2]);
}

#[cfg(test)]
#[derive(Clone)]
struct Composite {
    r: Map<u8, u8, 1>,
}

#[test]
fn insert_composite() {
    let mut m: Map<u8, Composite, 8> = Map::new();
    let c = Composite { r: Map::new() };
    m.insert(1, c);
    assert_eq!(0, m.into_iter().next().unwrap().1.r.len());
}

#[derive(Clone, Copy)]
struct Bar {}

#[test]
fn large_map_in_heap() {
    let m: Box<Map<u64, [u64; 10], 10>> = Box::new(Map::new());
    assert_eq!(0, m.len());
}

#[test]
fn clears_it_up() {
    let mut m: Map<&str, i32, 10> = Map::new();
    m.insert("one", 42);
    m.clear();
    assert_eq!(0, m.len());
}

#[test]
fn retain_test() {
    let vec: Vec<(i32, i32)> = (0..8).map(|x| (x, x * 10)).collect();
    let mut m: Map<i32, i32, 10> = Map::from_iter(vec);
    assert_eq!(m.len(), 8);
    m.retain(|&k, _| k < 6);
    assert_eq!(m.len(), 6);
    m.retain(|_, &v| v > 30);
    assert_eq!(m.len(), 2);
}

#[test]
#[ignore]
fn insert_many_and_remove() {
    let mut m: Map<usize, u64, 4> = Map::new();
    for _ in 0..2 {
        let cap = m.capacity();
        for i in 0..cap {
            println!("insert({i})");
            m.insert(i, 256);
            println!("remove({i})");
            m.remove(&i);
        }
    }
}
