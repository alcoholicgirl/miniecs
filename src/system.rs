use ecs_macros::all_tuples;
use std::marker::PhantomData;

use crate::{ComponentStorage, ComponentType, Fetch};

pub trait System: Sync + Send {
    fn run(&mut self, storage: &mut ComponentStorage);
}

impl<T, F> System for SystemObject<T, F>
where
    F: SystemFn<T>,
    T: Fetch,
{
    fn run(&mut self, storage: &mut ComponentStorage) {
        let all_entities = storage.all_entities();
        for entity in &all_entities {
            if let Some(params) = T::fetch(storage, *entity) {
                self.system.run(params);
            }
        }
        
        // buggy, to be fixed
        // let all_idents = T::idents();
        // for param in T::fetch_all(storage) {
        //     self.system.run(param);
        // }
    }
}

/// A function whose parameters are all component references (immutable or mutable).  
/// The scheduler will prioritize systems with higher priority first.
/// A system with priority 0 will be scheduled before another with priority 1.
pub struct SystemObject<T, F>
where
    F: SystemFn<T>,
    T: Fetch,
{
    system: F,
    phantom: PhantomData<T>,
    pub priority: i32,
}

impl<T: Fetch, F: SystemFn<T>> SystemObject<T, F> {
    pub fn set_priority(&mut self, priority: i32) {
        self.priority = priority;
    }
}

impl<T: Fetch, F: SystemFn<T>> From<F> for SystemObject<T, F> {
    fn from(value: F) -> Self {
        Self {
            system: value,
            phantom: PhantomData,
            priority: 1,
        }
    }
}


pub trait SystemFn<T>
where
    T: Fetch,
{
    fn run(&mut self, params: T);
}

macro_rules! all_fetches {
    ($($fetch: ident), *) => {
        #[allow(unused_parens)]
        impl<F, $($fetch), *> SystemFn<($($fetch), *)> for F
        where
            F : FnMut($($fetch), *) + 'static,
            $($fetch : ComponentType, $fetch :: PROTOTYPE : ComponentType), *,
            ($($fetch), *): Fetch
        {
            fn run(&mut self, ($($fetch), *) : ($($fetch), *)){
                (self)($($fetch), *);
            }
        }
    };
}

all_tuples!(all_fetches, 25, P);

unsafe impl<T: Fetch, F: SystemFn<T>> Sync for SystemObject<T, F> {}
unsafe impl<T: Fetch, F: SystemFn<T>> Send for SystemObject<T, F> {}
