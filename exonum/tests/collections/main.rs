// Copyright 2018 The Exonum Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Tests that compare exonum collections and corresponding rust types using proptest.

extern crate exonum;
#[macro_use]
extern crate proptest;
extern crate modifier;

use modifier::Modifier;
use std::collections::HashMap;
use std::collections::HashSet;

macro_rules! proptest_compare_collections {
    ($name:ident, $collection:ident, $reference:ident) => {
        proptest! {
            #[test]
            fn $name(ref actions in vec(generate_action(), 1..ACTIONS_MAX_LEN)) {
                let db = MemoryDB::new();

                let mut fork = db.fork();
                let mut reference = $reference::new();

                for action in actions {
                    match action {
                        MergeFork => {
                            db.merge(fork.into_patch()).unwrap();
                            fork = db.fork();
                        }
                        _ => {
                            let mut collection = $collection::new("test", &mut fork);
                            action.clone().modify(&mut collection);
                            action.clone().modify(&mut reference);
                            compare_collections(&collection, &reference)?;
                        }
                    }
                }
                db.merge(fork.into_patch()).unwrap();

                let mut fork = db.fork();
                let collection = $collection::new("test", &mut fork);
                compare_collections(&collection, &reference)?;
            }
        }
    };
}

mod key_set_index;
mod list_index;
mod map_index;
mod proof_list_index;
mod proof_map_index;
mod value_set_index;

// Max size of the generated sequence of actions.
const ACTIONS_MAX_LEN: usize = 100;

#[derive(Debug, Clone)]
enum ListAction<V> {
    Push(V),
    Pop,
    Extend(Vec<V>),
    // Applied with argument modulo `collection.len()`.
    Truncate(u64),
    // Applied to index modulo `collection.len()`.
    Set(u64, V),
    Clear,
    MergeFork,
}

#[derive(Debug, Clone)]
enum MapAction<K, V> {
    // Should be applied to a small subset of keys (like modulo 8 for int).
    Put(K, V),
    // Should be applied to a small subset of keys (like modulo 8 for int).
    Remove(K),
    Clear,
    MergeFork,
}

#[derive(Debug, Clone)]
enum SetAction<V> {
    // Should be applied to a small subset of values (like modulo 8 for int).
    Put(V),
    // Should be applied to a small subset of values (like modulo 8 for int).
    Remove(V),
    Clear,
    MergeFork,
}

impl<V> Modifier<Vec<V>> for ListAction<V> {
    fn modify(self, list: &mut Vec<V>) {
        match self {
            ListAction::Push(val) => {
                list.push(val);
            }
            ListAction::Pop => {
                list.pop();
            }
            ListAction::Extend(vec) => {
                list.extend(vec);
            }
            ListAction::Truncate(size) => {
                let len = list.len();
                if len > 0 {
                    list.truncate(size as usize % len);
                }
            }
            ListAction::Set(idx, val) => {
                let len = list.len();
                if len > 0 {
                    list[idx as usize % len] = val;
                }
            }
            ListAction::Clear => {
                list.clear();
            }
            _ => unreachable!(),
        }
    }
}

impl<V> Modifier<HashMap<u8, V>> for MapAction<u8, V> {
    fn modify(self, map: &mut HashMap<u8, V>) {
        match self {
            MapAction::Put(k, v) => {
                let k = k % 8;
                map.insert(k, v);
            }
            MapAction::Remove(k) => {
                let k = k % 8;
                map.remove(&k);
            }
            MapAction::Clear => {
                map.clear();
            }
            _ => unreachable!(),
        }
    }
}

impl<V> Modifier<HashMap<[u8; 32], V>> for MapAction<[u8; 32], V> {
    fn modify(self, map: &mut HashMap<[u8; 32], V>) {
        match self {
            MapAction::Put(mut k, v) => {
                k[0] = k[0] % 8;
                map.insert(k, v);
            }
            MapAction::Remove(mut k) => {
                k[0] = k[0] % 8;
                map.remove(&k);
            }
            MapAction::Clear => {
                map.clear();
            }
            _ => unreachable!(),
        }
    }
}

impl Modifier<HashSet<u8>> for SetAction<u8> {
    fn modify(self, set: &mut HashSet<u8>) {
        match self {
            SetAction::Put(mut v) => {
                v = v % 8;
                set.insert(v);
            }
            SetAction::Remove(mut v) => {
                v = v % 8;
                set.remove(&v);
            }
            SetAction::Clear => set.clear(),
            _ => unreachable!(),
        }
    }
}
