use std::sync::{Arc, Mutex};

use ahash::AHashMap;

use crate::{system::*, ComponentStorage, Fetch};

pub struct Scheduler {
    storage: AHashMap<usize, Box<dyn System>>,
    next_system_id: usize,
    priorities: Vec<(usize, i32)>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            next_system_id: 0,
            storage: AHashMap::new(),
            priorities: Vec::new(),
        }
    }

    pub fn push<T: Fetch + 'static, F: SystemFn<T> + 'static>(
        &mut self,
        system: F,
    ) -> usize {
        let system: SystemObject<T, F> = system.into();
        self.next_system_id += 1;
        let identifier = self.next_system_id;
        let priority = system.priority;

        self.priorities.push((identifier, priority));
        self.priorities.sort_by(|a, b| a.1.cmp(&b.1)); // sort according to priorities

        self.storage.insert(self.next_system_id, Box::new(system));
        identifier
    }

    pub fn drop(&mut self, system_ident: usize) -> Option<Box<dyn System>> {
        self.priorities.remove(
            (&self.priorities)
                .into_iter()
                .position(|x| x.0 == system_ident)
                .unwrap(),
        );
        self.storage.remove(&system_ident)
    }

    pub fn schedule(&mut self, storage_handle: Arc<Mutex<ComponentStorage>>) {
        for (sys_ident, _) in &self.priorities {
            let system = self.storage.get_mut(&sys_ident).unwrap();
            if let Ok(mut handle) = storage_handle.try_lock() {
                system.run(&mut *handle);
            }
        }
    }

    pub fn system_count(&self) -> usize {
        self.storage.len()
    }
}
