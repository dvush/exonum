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

//! Property testing for proof map index as a rust collection.

extern crate exonum;
#[macro_use]
extern crate proptest;

use exonum::storage::{Database, Fork, MemoryDB, ProofMapIndex};
use proptest::{collection::vec, num, prelude::*, strategy};
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum Action {
    //Applied to key where key[0] is modulo 8
    Put([u8; 32], i32),
    //Applied to key where key[0] is modulo 8
    Remove([u8; 32]),
    Clear,
    MergeFork,
}

impl Action {
    fn apply_map(
        &self,
        map: &mut ProofMapIndex<&mut Fork, [u8; 32], i32>,
        ref_map: &mut HashMap<[u8; 32], i32>,
    ) {
        match *self {
            Action::Put(mut k, v) => {
                k[0] = k[0] % 8;
                map.put(&k, v);
                ref_map.insert(k, v);
            }
            Action::Remove(mut k) => {
                k[0] = k[0] % 8;
                map.remove(&k);
                ref_map.remove(&k);
            }
            Action::Clear => {
                map.clear();
                ref_map.clear();
            }
            _ => unreachable!(),
        }
    }
}

proptest!{
    #[test]
    fn proptest_proof_map_index_to_rust_map(ref actions in
                     vec( prop_oneof![
                         (num::u8::ANY, num::i32::ANY).prop_map(|(i, v)|{
                             let mut key = [0u8;32];
                             key[0] = i;
                             Action::Put(key,v)
                         }),
                         num::u8::ANY.prop_map(|i| {
                             let mut key = [0u8;32];
                             key[0] = i;
                             Action::Remove(key)}),
                         strategy::Just(Action::Clear),
                         strategy::Just(Action::MergeFork),
                     ] , 1..10) ) {
        let db = MemoryDB::new();

        let mut fork = db.fork();
        let mut ref_map : HashMap<[u8; 32], i32> = HashMap::new();

        for action in actions {
            match action {
                Action::MergeFork => {
                    db.merge(fork.into_patch()).unwrap();
                    fork = db.fork();
                },
                _ => {
                    let mut map = ProofMapIndex::<_, [u8; 32], i32>::new("test", &mut fork);
                    action.apply_map(&mut map, &mut ref_map);
                }
            }
        }
        db.merge(fork.into_patch()).unwrap();

        let snapshot = db.snapshot();
        let map_index = ProofMapIndex::<_, [u8; 32], i32>::new("test", &snapshot);

        for k in ref_map.keys() {
            prop_assert!(map_index.contains(k));
        }
        for (k,v) in map_index.iter() {
            prop_assert_eq!(Some(&v), ref_map.get(&k));
        }
    }
}