use cache::Cache;

mod cache {
    use std::{collections::VecDeque, fmt};

    struct CacheEntry {
        key: String,
        value: String,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub enum CacheError {
        EmptyKey,
        DisabledCache,
    }

    impl fmt::Display for CacheError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(match self {
                Self::EmptyKey => "empty_key",
                Self::DisabledCache => "disabled_cache",
            })
        }
    }

    pub struct Cache {
        entries: VecDeque<CacheEntry>,
        capacity: usize,
    }

    impl Cache {
        pub fn new(capacity: usize) -> Self {
            Self {
                entries: VecDeque::new(),
                capacity,
            }
        }

        pub fn put(&mut self, key: String, value: String) -> Result<(), CacheError> {
            if key.is_empty() {
                return Err(CacheError::EmptyKey);
            }
            if self.capacity == 0 {
                return Err(CacheError::DisabledCache);
            }

            if let Some(entry) = self.entries.iter_mut().find(|entry| entry.key == key) {
                entry.value = value;
            } else {
                if self.entries.len() >= self.capacity {
                    self.entries.pop_front();
                }
                self.entries.push_back(CacheEntry { key, value });
            }

            Ok(())
        }

        /// Borrows the stored value from this cache, independently of the lookup key.
        /// The reference cannot outlive the cache. While it will still be used,
        /// `put` cannot mutably borrow the cache to replace or evict its entries.
        pub fn get(&self, key: &str) -> Option<&str> {
            self.entries
                .iter()
                .find(|entry| entry.key == key)
                .map(|entry| entry.value.as_str())
        }
    }
}

fn main() {
    let mut cache = Cache::new(2);

    cache.put("a".into(), "one".into()).unwrap();
    cache.put("b".into(), "two".into()).unwrap();
    cache.put("a".into(), "updated".into()).unwrap();
    cache.put("c".into(), "three".into()).unwrap();

    println!("a={:?}", cache.get("a"));
    println!("b={:?}", cache.get("b"));
    println!("c={:?}", cache.get("c"));
}

#[cfg(test)]
mod tests {
    use super::{Cache, cache::CacheError};

    #[test]
    fn new_cache_starts_empty() {
        let cache = Cache::new(2);

        assert_eq!(cache.get("a"), None);
        assert_eq!(cache.get(""), None);
    }

    #[test]
    fn read_borrows_stored_value_independently_of_lookup_key() {
        let mut cache = Cache::new(2);
        cache.put("a".into(), "one".into()).unwrap();
        assert_eq!(cache.get("missing"), None);
        assert_eq!(cache.get(""), None);

        let value = {
            let key = String::from("a");
            cache.get(&key).unwrap()
        };

        assert_eq!(value, "one");
        assert!(std::ptr::eq(value, cache.get("a").unwrap()));
    }

    #[test]
    fn empty_key_takes_priority_when_cache_is_disabled() {
        let mut cache = Cache::new(0);

        let empty_key = cache.put("".into(), "value".into()).unwrap_err();
        assert_eq!(empty_key, CacheError::EmptyKey);
        assert_eq!(empty_key.to_string(), "empty_key");
        assert_eq!(cache.get(""), None);
    }

    #[test]
    fn zero_capacity_rejects_nonempty_keys() {
        let mut cache = Cache::new(0);

        let disabled_cache = cache.put("a".into(), "value".into()).unwrap_err();
        assert_eq!(disabled_cache, CacheError::DisabledCache);
        assert_eq!(disabled_cache.to_string(), "disabled_cache");
        assert_eq!(cache.get("a"), None);
        assert_eq!(cache.get(""), None);
    }

    #[test]
    fn failed_write_preserves_contents_and_fifo_order() {
        let mut cache = Cache::new(2);
        cache.put("a".into(), "one".into()).unwrap();
        cache.put("b".into(), "two".into()).unwrap();

        assert_eq!(
            cache.put("".into(), "invalid".into()),
            Err(CacheError::EmptyKey)
        );
        assert_eq!(cache.get("a"), Some("one"));
        assert_eq!(cache.get("b"), Some("two"));
        assert_eq!(cache.get(""), None);

        cache.put("c".into(), "three".into()).unwrap();
        assert_eq!(cache.get("a"), None);
        assert_eq!(cache.get("b"), Some("two"));
        assert_eq!(cache.get("c"), Some("three"));
    }

    #[test]
    fn updates_do_not_duplicate_entries_or_change_fifo_order() {
        let mut cache = Cache::new(2);
        cache.put("a".into(), "one".into()).unwrap();
        cache.put("a".into(), "updated".into()).unwrap();
        cache.put("b".into(), "two".into()).unwrap();
        assert_eq!(cache.get("a"), Some("updated"));
        assert_eq!(cache.get("b"), Some("two"));

        cache.put("a".into(), "latest".into()).unwrap();
        assert_eq!(cache.get("a"), Some("latest"));
        assert_eq!(cache.get("b"), Some("two"));

        cache.put("c".into(), "three".into()).unwrap();
        assert_eq!(cache.get("a"), None);
        assert_eq!(cache.get("b"), Some("two"));
        assert_eq!(cache.get("c"), Some("three"));
    }

    #[test]
    fn capacity_one_evicts_the_oldest_entry() {
        let mut cache = Cache::new(1);
        cache.put("a".into(), "one".into()).unwrap();
        cache.put("b".into(), "two".into()).unwrap();

        assert_eq!(cache.get("a"), None);
        assert_eq!(cache.get("b"), Some("two"));
    }

    #[test]
    fn inserting_into_a_full_cache_evicts_the_oldest_entry() {
        let mut cache = Cache::new(2);
        cache.put("a".into(), "one".into()).unwrap();
        cache.put("b".into(), "two".into()).unwrap();
        cache.put("c".into(), "three".into()).unwrap();

        assert_eq!(cache.get("a"), None);
        assert_eq!(cache.get("b"), Some("two"));
        assert_eq!(cache.get("c"), Some("three"));
    }

    #[test]
    fn reading_an_entry_does_not_change_fifo_order() {
        let mut cache = Cache::new(2);
        cache.put("a".into(), "one".into()).unwrap();
        cache.put("b".into(), "two".into()).unwrap();
        assert_eq!(cache.get("a"), Some("one"));

        cache.put("c".into(), "three".into()).unwrap();
        assert_eq!(cache.get("a"), None);
        assert_eq!(cache.get("b"), Some("two"));
        assert_eq!(cache.get("c"), Some("three"));
    }

    #[test]
    fn values_may_be_empty() {
        let mut cache = Cache::new(2);
        cache.put("a".into(), "".into()).unwrap();

        assert_eq!(cache.get("a"), Some(""));
    }

    #[test]
    fn keys_are_case_sensitive() {
        let mut cache = Cache::new(2);
        cache.put("a".into(), "lowercase".into()).unwrap();
        cache.put("A".into(), "uppercase".into()).unwrap();

        assert_eq!(cache.get("a"), Some("lowercase"));
        assert_eq!(cache.get("A"), Some("uppercase"));
    }

    #[test]
    fn keys_are_not_trimmed() {
        let mut cache = Cache::new(2);
        cache.put("a".into(), "plain".into()).unwrap();
        cache.put(" a ".into(), "padded".into()).unwrap();

        assert_eq!(cache.get("a"), Some("plain"));
        assert_eq!(cache.get(" a "), Some("padded"));
        assert_eq!(cache.get(" a"), None);
        assert_eq!(cache.get("a "), None);
    }

    #[test]
    fn repeated_evictions_preserve_fifo_order() {
        let mut cache = Cache::new(2);
        for key in ["a", "b", "c", "d", "e", "f"] {
            cache.put(key.into(), key.into()).unwrap();
        }
        for evicted in ["a", "b", "c", "d"] {
            assert_eq!(cache.get(evicted), None);
        }
        assert_eq!(cache.get("e"), Some("e"));
        assert_eq!(cache.get("f"), Some("f"));

        cache.put("e".into(), "updated".into()).unwrap();
        assert_eq!(cache.get("e"), Some("updated"));
        cache.put("g".into(), "g".into()).unwrap();
        assert_eq!(cache.get("e"), None);
        assert_eq!(cache.get("f"), Some("f"));
        assert_eq!(cache.get("g"), Some("g"));
    }
}
