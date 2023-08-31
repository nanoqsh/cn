use {
    crate::db::Access,
    std::{
        collections::{hash_map::Entry, HashMap},
        rc::Rc,
    },
};

type Key = Rc<str>;

pub struct Cache {
    db: Access,
    map: HashMap<Key, Option<String>>,
    keys: Vec<Key>,
}

impl Cache {
    pub fn new(size: usize, db: Access) -> Self {
        Self {
            db,
            map: HashMap::with_capacity(size),
            keys: Vec::with_capacity(size),
        }
    }

    pub async fn fetch(&mut self, key: String) -> Option<String> {
        let key = Rc::from(key);
        match self.map.entry(Rc::clone(&key)) {
            Entry::Occupied(en) => {
                let link = en.get().as_ref().cloned();
                self.popup(&key);
                link
            }
            Entry::Vacant(en) => {
                let link = self.db.load(key.as_ref().to_owned()).await;
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

    fn popup(&mut self, key: &Rc<str>) {
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
        let mut cache = Cache::new(0, db().await);

        let v = cache.fetch(String::from("ak")).await;
        assert_eq!(v, Some(String::from("av")));
        assert_eq!(cache.map.len(), 0);
        assert_eq!(cache.keys, vec![]);

        let v = cache.fetch(String::from("bk")).await;
        assert_eq!(v, Some(String::from("bv")));
        assert_eq!(cache.map.len(), 0);
        assert_eq!(cache.keys, vec![]);
    }

    #[tokio::test]
    async fn cache_1() {
        let mut cache = Cache::new(1, db().await);

        // Fetch first key
        let v = cache.fetch(String::from("ak")).await;
        assert_eq!(v, Some(String::from("av")));
        assert_eq!(cache.map.len(), 1);
        assert_eq!(cache.keys, vec![Rc::from("ak")]);

        // Fetch this key again
        let v = cache.fetch(String::from("ak")).await;
        assert_eq!(v, Some(String::from("av")));
        assert_eq!(cache.map.len(), 1);
        assert_eq!(cache.keys, vec![Rc::from("ak")]);

        // Fetch second key
        let v = cache.fetch(String::from("bk")).await;
        assert_eq!(v, Some(String::from("bv")));
        assert_eq!(cache.map.len(), 1);
        assert_eq!(cache.keys, vec![Rc::from("bk")]);
    }

    #[tokio::test]
    async fn cache_2() {
        let mut cache = Cache::new(2, db().await);

        // Fetch first key
        let v = cache.fetch(String::from("ak")).await;
        assert_eq!(v, Some(String::from("av")));
        assert_eq!(cache.map.len(), 1);
        assert_eq!(cache.keys, vec![Rc::from("ak")]);

        // Fetch second key, first moves to left
        let v = cache.fetch(String::from("bk")).await;
        assert_eq!(v, Some(String::from("bv")));
        assert_eq!(cache.map.len(), 2);
        assert_eq!(cache.keys, vec![Rc::from("ak"), Rc::from("bk")]);

        // Fetch next, prev moves to left
        let v = cache.fetch(String::from("ck")).await;
        assert_eq!(v, Some(String::from("cv")));
        assert_eq!(cache.map.len(), 2);
        assert_eq!(cache.keys, vec![Rc::from("bk"), Rc::from("ck")]);

        // Fetch cached key, it pops up
        let v = cache.fetch(String::from("bk")).await;
        assert_eq!(v, Some(String::from("bv")));
        assert_eq!(cache.map.len(), 2);
        assert_eq!(cache.keys, vec![Rc::from("ck"), Rc::from("bk")]);

        // Fetch next key, prev moves to left
        let v = cache.fetch(String::from("dk")).await;
        assert_eq!(v, None);
        assert_eq!(cache.map.len(), 2);
        assert_eq!(cache.keys, vec![Rc::from("bk"), Rc::from("dk")]);

        // Fetch this key again, it keeps position
        let v = cache.fetch(String::from("dk")).await;
        assert_eq!(v, None);
        assert_eq!(cache.map.len(), 2);
        assert_eq!(cache.keys, vec![Rc::from("bk"), Rc::from("dk")]);
    }

    #[tokio::test]
    async fn cache_3() {
        let mut cache = Cache::new(3, db().await);

        let v = cache.fetch(String::from("ak")).await;
        assert_eq!(v, Some(String::from("av")));
        assert_eq!(cache.map.len(), 1);
        assert_eq!(cache.keys, vec![Rc::from("ak")]);

        let v = cache.fetch(String::from("bk")).await;
        assert_eq!(v, Some(String::from("bv")));
        assert_eq!(cache.map.len(), 2);
        assert_eq!(cache.keys, vec![Rc::from("ak"), Rc::from("bk")]);

        let v = cache.fetch(String::from("ck")).await;
        assert_eq!(v, Some(String::from("cv")));
        assert_eq!(cache.map.len(), 3);
        assert_eq!(
            cache.keys,
            vec![Rc::from("ak"), Rc::from("bk"), Rc::from("ck")]
        );
    }

    async fn db() -> Access {
        let (db, service) = crate::db::test().expect("test db");
        tokio::spawn(service.run());
        db.store(String::from("ak"), String::from("av")).await;
        db.store(String::from("bk"), String::from("bv")).await;
        db.store(String::from("ck"), String::from("cv")).await;
        db
    }
}
