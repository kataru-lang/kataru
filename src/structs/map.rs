use crate::traits::Merge;
use crate::{error::Result, traits::MoveValues};
use std::collections::BTreeMap;

pub use std::collections::btree_map::Entry;

pub type Map<K, V> = BTreeMap<K, V>;

fn copy_keys<V>(map: &Map<String, V>) -> Vec<String> {
    let mut keys: Vec<String> = Vec::with_capacity(map.len());
    for key in map.keys() {
        keys.push(key.to_string());
    }
    keys
}

impl<V> MoveValues for Map<String, V> {
    fn move_values(other: &mut Self) -> Result<Self> {
        let keys = copy_keys(other);
        let mut map = Self::new();
        for key in keys {
            let value = other.remove(&key).unwrap();
            map.insert(key, value);
        }
        Ok(map)
    }
}

impl<V> Merge for Map<String, V> {
    fn merge(&mut self, other: &mut Self) -> Result<()> {
        let keys = copy_keys(other);
        for key in keys {
            if !self.contains_key(&key) {
                let value = other.remove(&key).unwrap();
                self.insert(key, value);
            }
        }
        Ok(())
    }
}
