use ecs_macros::all_tuples;

use crate::{ComponentStorage, Entity};

pub trait ComponentType: Sync + Send {
    const IDENTIFIER: usize;
    const MUTABLE: bool;
    type PROTOTYPE;
}

pub trait ComponentInstance: Sync + Send {
    fn get_component_id(&self) -> usize;
}

pub trait Fetch
where
    Self: Sync + Send + Sized,
{
    fn fetch(storage: &mut ComponentStorage, entity: Entity) -> Option<Self>;
    fn idents() -> Vec<usize>;
    // fn fetch_all(storage: &mut ComponentStorage) -> impl Iterator<Item = Self>;
}

macro_rules! fetch_ref {
    ($p : ty, $map : expr, $arch : expr, $action : stmt) => {
        match $map.get(&<$p>::IDENTIFIER) {
            None => {$action},
            Some(key) => {
                let component = $arch.get(*key).unwrap();
                unsafe {
                    std::mem::transmute_copy::<&<$p as ComponentType>::PROTOTYPE, $p>(
                        &&*(component.as_ref() as *const dyn ComponentInstance
                            as *const <$p>::PROTOTYPE),
                    )
                }
            }
        }
    };
}

macro_rules! fetch_mut {
    ($p : ty, $map : expr, $arch : expr, $action : stmt) => {
        match $map.get(&<$p>::IDENTIFIER) {
            None => {$action},
            Some(key) => {
                let component = $arch.get_mut(*key).unwrap();
                unsafe {
                    std::mem::transmute_copy::<&mut <$p as ComponentType>::PROTOTYPE, $p>(
                        &&mut *(component.as_mut() as *mut dyn ComponentInstance
                            as *mut <$p>::PROTOTYPE),
                    )
                }
            }
        }
    };
}


// implement fetch trait for component tuples
macro_rules! impl_fetch {
    ($($a : ident), *) => {
        #[allow(unused_parens)]
        impl<$($a),*> Fetch for ($($a),*) where
        $($a : ComponentType ),*, $($a :: PROTOTYPE : ComponentType ),* {

            fn fetch(storage: &mut ComponentStorage, entity: Entity) -> Option<Self> {
                let (archetypes, id_map) = storage.get_archetype(entity);
                Some(($(
                    if $a :: MUTABLE {
                        fetch_mut!($a, id_map, archetypes, return None)
                    } else {
                        fetch_ref!($a, id_map, archetypes, return None)
                    }
                ), *))
            }
            
            fn idents() -> Vec<usize> {
                vec![$($a :: IDENTIFIER),*]
            }


            // This is not working as expected!!! to be fixed!!!
            // Once fixed, fetching might be somewhat faster
            // fn fetch_all(storage: &mut ComponentStorage) -> impl Iterator<Item = Self> {
            //     let (archetypes_all, id_map) = storage.get_archetype_iter();
            //     let mut result : Vec<Self> = Vec::new();
            //     for (entity_key, archetype) in archetypes_all {
            //         let new_arch = {
            //             ($(
            //                 if $a :: MUTABLE {
            //                     fetch_mut!($a, id_map.get_mut(&entity_key).unwrap(), archetype, break)
            //                 } else {
            //                     fetch_ref!($a, id_map.get_mut(&entity_key).unwrap(), archetype, break)
            //                 }
            //             ), *)
            //         };
            //         result.push(new_arch);
            //     }

            //     result.into_iter()
            // }
        }
    };
}

all_tuples!(impl_fetch, 25, P);