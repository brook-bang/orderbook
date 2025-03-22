use ahash::AHashMap as HashMap;
//use dashmap::DashMap as HashMap;
// use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug)]
pub struct SparseVec<K, V>
where
    K: Eq + Hash + Ord + Clone,
{
    data: HashMap<K, V>,
}

impl<K, V> Default for SparseVec<K, V>
where
    K: Eq + Hash + Ord + Clone,
{
    fn default() -> Self {
        SparseVec {
            data: HashMap::new(),
        }
    }
}

impl<K, V> SparseVec<K, V>
where
    K: Eq + Hash + Ord + Clone,
{
    pub fn with_capacity(capacity: usize) -> Self {
        SparseVec {
            data: HashMap::with_capacity(capacity),
        }
    }

    pub fn insert(&mut self, index: K, value: V) -> Option<V> {
        self.data.insert(index, value)
    }

    pub fn remove(&mut self, index: &K) -> Option<V> {
        self.data.remove(index)
    }

    pub fn get(&self, index: &K) -> Option<&V> {
        self.data.get(index)
    }

    pub fn get_mut(&mut self, index: &K) -> Option<&mut V> {
        self.data.get_mut(index)
    }

    pub fn max_index(&self) -> Option<K> {
        self.data.keys().max().cloned()
    }

    pub fn min_index(&self) -> Option<K> {
        self.data.keys().min().cloned()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.data.iter()
    }
}
