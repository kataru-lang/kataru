use linear_map::LinearMap;

use crate::traits::{CopyMerge, Merge};
use crate::{error::Result, traits::MoveValues};
use std::collections::HashMap;

pub use std::collections::btree_map::Entry;

pub type Map<K, V> = HashMap<K, V>;

fn copy_keys<V>(map: &Map<String, V>) -> Vec<String> {
    let mut keys: Vec<String> = Vec::with_capacity(map.len());
    for key in map.keys() {
        keys.push(key.to_string());
    }
    keys
}

fn copy_keys_linear_map<V>(map: &LinearMap<String, V>) -> Vec<String> {
    let mut keys: Vec<String> = Vec::with_capacity(map.len());
    for key in map.keys() {
        keys.push(key.to_string());
    }
    keys
}

impl<V> MoveValues for LinearMap<String, V> {
    fn move_values(other: &mut Self) -> Result<Self> {
        let keys = copy_keys_linear_map(other);
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
        for key in &keys {
            self.entry(key.clone())
                .or_insert_with(|| other.remove(key).unwrap());
        }
        Ok(())
    }
}

impl<V: Clone> CopyMerge for Map<String, V> {
    fn copy_merge(&self, other: &Self) -> Result<Self> {
        let mut merged = self.clone();
        for (key, value) in other {
            if !merged.contains_key(key) {
                merged.insert(key.to_string(), value.clone());
            }
        }
        Ok(merged)
    }
}

impl<V: Clone> CopyMerge for LinearMap<String, V> {
    fn copy_merge(&self, other: &Self) -> Result<Self> {
        let mut merged = self.clone();
        for (key, value) in other {
            if !merged.contains_key(key) {
                merged.insert(key.to_string(), value.clone());
            }
        }
        Ok(merged)
    }
}
