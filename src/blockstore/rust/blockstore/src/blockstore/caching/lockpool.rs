use anyhow::Result;
use owning_ref::OwningHandle;
use std::collections::hash_map::{Entry, HashMap};
use std::hash::Hash;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};

pub struct LockPool<K>
where
    K: Eq + PartialEq + Hash + Clone,
{
    currently_locked: Mutex<HashMap<K, Arc<Mutex<()>>>>,
}

impl<K> LockPool<K>
where
    K: Eq + PartialEq + Hash + Clone,
{
    pub fn new() -> Self {
        Self {
            currently_locked: Mutex::new(HashMap::new()),
        }
    }

    pub fn lock<'a>(&'a self, key: K) -> Guard<'a, K> {
        // TODO Return Result instead of expect()
        let mut currently_locked = self.currently_locked.lock().expect("Poisoned mutex");
        if let Some(mutex) = currently_locked.get_mut(&key).map(|a| Arc::clone(a)) {
            let guard = OwningHandle::new_with_fn(mutex, |mutex| unsafe {
                // TODO Return Result instead of expect()
                (*mutex).lock().expect("Poisoned mutex")
            });
            Guard { key, guard }
        } else {
            let insert_result = currently_locked.insert(key.clone(), Arc::new(Mutex::new(())));
            assert!(
                insert_result.is_none(),
                "We just checked that the entry doesn't exist, why does it exist now?"
            );
            let mutex = currently_locked
                .get_mut(&key)
                .map(|a| Arc::clone(a))
                .expect("We just inserted this");
            let guard = OwningHandle::new_with_fn(mutex, |mutex| unsafe {
                // TODO Return Result instead of expect()
                (*mutex).lock().expect("Poisoned mutex")
            });
            Guard { key, guard }
        }
    }
}

pub struct Guard<'a, K> {
    key: K,
    guard: OwningHandle<Arc<Mutex<()>>, MutexGuard<'a, ()>>,
}
