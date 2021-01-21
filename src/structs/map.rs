use crate::error::ParseError;
use crate::traits::Mergeable;
use std::collections::BTreeMap;

pub use std::collections::btree_map::Entry;

pub type Map<K, V> = BTreeMap<K, V>;

impl<V> Mergeable for Map<String, V> {
    fn merge(&mut self, other: &mut Self) -> Result<(), ParseError> {
        let keys = Self::copy_keys(other);
        for key in &keys {
            self.entry(key.clone())
                .or_insert(other.remove(key).unwrap());
        }
        Ok(())
    }
}
