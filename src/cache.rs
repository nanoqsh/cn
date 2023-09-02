use {
    crate::db::Access,
    std::{
        collections::{hash_map::Entry, HashMap},
        future::Future,
        sync::Arc,
    },
    tokio::sync::RwLock,
};

#[derive(Clone)]
pub struct Cache {
    db: Access,
    lru: Arc<RwLock<Lru>>,
}

impl Cache {
    pub fn new(size: usize, db: Access) -> Self {
        Self {
            db,
            lru: Arc::new(RwLock::new(Lru::new(size))),
        }
    }

    pub async fn store(&self, key: String, link: String) {
        self.db.store(key, link).await;
    }

    pub async fn fetch(&self, key: String) -> Option<String> {
        let mut lru = self.lru.write().await;
        lru.fetch(key, |key| self.db.load(key)).await
    }
}

type Key = Arc<str>;

struct Lru {
    map: HashMap<Key, Option<String>>,
    keys: Vec<Key>,
}

impl Lru {
    fn new(size: usize) -> Self {
        Self {
            map: HashMap::with_capacity(size),
            keys: Vec::with_capacity(size),
        }
    }

    async fn fetch<F, R>(&mut self, key: String, f: F) -> Option<String>
    where
        F: FnOnce(String) -> R,
        R: Future<Output = Option<String>>,
    {
        let key = Key::from(key);
        match self.map.entry(Key::clone(&key)) {
            Entry::Occupied(en) => {
                let link = en.get().as_ref().cloned();
                self.lift_up(&key);
                link
            }
            Entry::Vacant(en) => {
                let link = f(key.as_ref().to_owned()).await;
                if self.keys.capacity() == 0 {
                } else if self.keys.len() == self.keys.capacity() {
                    let removed = self.keys.remove(0);
                    self.keys.push(key);
                    en.insert(link.clone());
                    self.map.remove(&removed);
                } else {
                    en.insert(link.clone());
                    self.keys.push(key);
                }

                link
            }
        }
    }

    fn lift_up(&mut self, key: &Key) {
        let index = self
            .keys
            .iter()
            .rev()
            .position(|k| k == key)
            .expect("find index");

        let index = self.keys.len() - index - 1;
        self.keys[index..].rotate_left(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cache_0() {
        let mut cache = Lru::new(0);

        let v = cache.test_fetch(String::from("ak")).await;
        assert_eq!(v, Some(String::from("av")));
        assert_eq!(cache.map.len(), 0);
        assert_eq!(cache.keys, vec![]);

        let v = cache.test_fetch(String::from("bk")).await;
        assert_eq!(v, Some(String::from("bv")));
        assert_eq!(cache.map.len(), 0);
        assert_eq!(cache.keys, vec![]);
    }

    #[tokio::test]
    async fn cache_1() {
        let mut cache = Lru::new(1);

        // Fetch first key
        let v = cache.test_fetch(String::from("ak")).await;
        assert_eq!(v, Some(String::from("av")));
        assert_eq!(cache.map.len(), 1);
        assert_eq!(cache.keys, vec![Key::from("ak")]);

        // Fetch this key again
        let v = cache.test_fetch(String::from("ak")).await;
        assert_eq!(v, Some(String::from("av")));
        assert_eq!(cache.map.len(), 1);
        assert_eq!(cache.keys, vec![Key::from("ak")]);

        // Fetch second key
        let v = cache.test_fetch(String::from("bk")).await;
        assert_eq!(v, Some(String::from("bv")));
        assert_eq!(cache.map.len(), 1);
        assert_eq!(cache.keys, vec![Key::from("bk")]);
    }

    #[tokio::test]
    async fn cache_2() {
        let mut cache = Lru::new(2);

        // Fetch first key
        let v = cache.test_fetch(String::from("ak")).await;
        assert_eq!(v, Some(String::from("av")));
        assert_eq!(cache.map.len(), 1);
        assert_eq!(cache.keys, vec![Key::from("ak")]);

        // Fetch second key, first moves to left
        let v = cache.test_fetch(String::from("bk")).await;
        assert_eq!(v, Some(String::from("bv")));
        assert_eq!(cache.map.len(), 2);
        assert_eq!(cache.keys, vec![Key::from("ak"), Key::from("bk")]);

        // Fetch next, prev moves to left
        let v = cache.test_fetch(String::from("ck")).await;
        assert_eq!(v, Some(String::from("cv")));
        assert_eq!(cache.map.len(), 2);
        assert_eq!(cache.keys, vec![Key::from("bk"), Key::from("ck")]);

        // Fetch cached key, it lifts up
        let v = cache.test_fetch(String::from("bk")).await;
        assert_eq!(v, Some(String::from("bv")));
        assert_eq!(cache.map.len(), 2);
        assert_eq!(cache.keys, vec![Key::from("ck"), Key::from("bk")]);

        // Fetch next key, prev moves to left
        let v = cache.test_fetch(String::from("dk")).await;
        assert_eq!(v, None);
        assert_eq!(cache.map.len(), 2);
        assert_eq!(cache.keys, vec![Key::from("bk"), Key::from("dk")]);

        // Fetch this key again, it keeps position
        let v = cache.test_fetch(String::from("dk")).await;
        assert_eq!(v, None);
        assert_eq!(cache.map.len(), 2);
        assert_eq!(cache.keys, vec![Key::from("bk"), Key::from("dk")]);
    }

    #[tokio::test]
    async fn cache_3() {
        let mut cache = Lru::new(3);

        let v = cache.test_fetch(String::from("ak")).await;
        assert_eq!(v, Some(String::from("av")));
        assert_eq!(cache.map.len(), 1);
        assert_eq!(cache.keys, vec![Key::from("ak")]);

        let v = cache.test_fetch(String::from("bk")).await;
        assert_eq!(v, Some(String::from("bv")));
        assert_eq!(cache.map.len(), 2);
        assert_eq!(cache.keys, vec![Key::from("ak"), Key::from("bk")]);

        let v = cache.test_fetch(String::from("ck")).await;
        assert_eq!(v, Some(String::from("cv")));
        assert_eq!(cache.map.len(), 3);
        assert_eq!(
            cache.keys,
            vec![Key::from("ak"), Key::from("bk"), Key::from("ck")]
        );
    }

    impl Lru {
        async fn test_fetch(&mut self, key: String) -> Option<String> {
            self.fetch(key, |key| async move {
                match key.as_str() {
                    "ak" => Some(String::from("av")),
                    "bk" => Some(String::from("bv")),
                    "ck" => Some(String::from("cv")),
                    _ => None,
                }
            })
            .await
        }
    }
}
