use std::collections::{HashMap, VecDeque};

#[derive(Debug, Default, Clone)]
pub struct LRUCache<K, V> {
    capacity: usize,
    cache_map: HashMap<K, usize>,
    cache_deque: VecDeque<(K, V)>,
}

impl<K, V> LRUCache<K, V>
where
    K: Eq + std::hash::Hash + Clone,
{
    fn new(capacity: usize) -> Self {
        LRUCache {
            capacity,
            cache_map: HashMap::new(),
            cache_deque: VecDeque::new(),
        }
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        if let Some(&index) = self.cache_map.get(key) {
            let (key, value) = match self.cache_deque.remove(index) {
                Some((key, value)) => (key, value),
                None => return None,
            };
            self.cache_deque.push_front((key.clone(), value));
            self.cache_map.insert(key, 0); // Update index to front
            Some(&self.cache_deque[0].1)
        } else {
            None
        }
    }

    fn put(&mut self, key: K, value: V) {
        if self.cache_map.contains_key(&key) {
            let index = self.cache_map[&key];
            self.cache_deque.remove(index);
            self.cache_deque.push_front((key.clone(), value));
            self.cache_map.insert(key, 0); // Update index to front
        } else {
            if self.cache_deque.len() >= self.capacity {
                let last_key = self.cache_deque.pop_back().unwrap().0;
                self.cache_map.remove(&last_key);
            }
            self.cache_deque.push_front((key.clone(), value));
            self.cache_map.insert(key, 0); // Update index to front
        }
    }
}
