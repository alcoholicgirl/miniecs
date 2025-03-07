use crate::{ComponentInstance, ComponentType};
use ahash::AHashMap;
use slotmap::{DefaultKey, SlotMap};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy)]
pub struct Entity(DefaultKey);

pub struct ComponentStorage {
    /// This stores archetypes corresponding to entities
    pub archetypes: SlotMap<DefaultKey, SlotMap<DefaultKey, Box<dyn ComponentInstance>>>,

    /// This map is used to look up the entry of a specific component
    /// within an archetype of a given entity using the component identifier.  
    /// `component_id_map.get(&some_entity.0)`
    pub component_id_map: AHashMap<DefaultKey, AHashMap<usize, DefaultKey>>,
}

pub struct World {
    storage_handle: Arc<Mutex<ComponentStorage>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            storage_handle: Arc::new(Mutex::new(ComponentStorage::new())),
        }
    }

    pub fn handle(&self) -> Arc<Mutex<ComponentStorage>> {
        self.storage_handle.clone()
    }

    /// Spawn a new entity
    pub fn spawn(&mut self) -> Entity {
        let new_ent = self
            .storage_handle
            .try_lock()
            .unwrap()
            .archetypes
            .insert(SlotMap::new());
        self.storage_handle
            .try_lock()
            .unwrap()
            .component_id_map
            .insert(new_ent, AHashMap::new());

        let entity = Entity(new_ent);
        entity
    }

    /// Kill an existing entity
    pub fn kill(&mut self, entity: Entity) -> Result<(), &str> {
        let handle = self.storage_handle.clone();
        let storage = handle.try_lock();
        if let Ok(mut storage) = storage {
            if storage.archetypes.contains_key(entity.0) {
                storage.archetypes.remove(entity.0);
            } else {
                return Err("entity not found in archetypes map");
            }
            if storage.component_id_map.contains_key(&entity.0) {
                storage.component_id_map.remove(&entity.0);
            } else {
                return Err("entity not found in id map");
            }
        }

        Ok(())
    }

    /// Adds a component to an existing entity.
    /// Components should be thread-safe.
    pub fn add_component(
        &mut self,
        entity: Entity,
        component: impl ComponentInstance + 'static,
    ) -> &mut Self {
        if !self
            .storage_handle
            .try_lock()
            .as_ref()
            .unwrap()
            .archetypes
            .contains_key(entity.0)
        {
            panic!("entity does not exist");
        }

        let component = Box::new(component);
        // Add components
        let id = component.get_component_id();
        let storage_lock = self.storage_handle.clone();
        let storage = &mut storage_lock.try_lock().unwrap();
        let components = storage.archetypes.get_mut(entity.0).unwrap();

        // mapping
        let component_id = component.get_component_id();
        let key = components.insert(component);
        storage
            .component_id_map
            .get_mut(&entity.0)
            .unwrap()
            .insert(id, key);

        self
    }

    /// Removes a component from an entity and returns it.   
    /// Returns None if the entity does not exist or does not have the specified component.
    pub fn take_component<T: ComponentType>(
        &mut self,
        entity: Entity,
    ) -> Option<Box<dyn ComponentInstance>> {
        let handle = self.storage_handle.clone();
        let mut storage = handle.try_lock().unwrap();
        if !storage.archetypes.contains_key(entity.0) {
            return None;
        }

        let (archetype, id_map) = storage.get_archetype(entity);
        if let Some(key) = id_map.get(&T::IDENTIFIER) {
            let component = archetype.remove(*key).unwrap();
            id_map.remove(&T::IDENTIFIER);
            Some(component)
        } else {
            None
        }
    }

    pub fn get_handle(&self) -> Arc<Mutex<ComponentStorage>>{
        self.storage_handle.clone()
    } 
}

impl ComponentStorage {
    pub fn new() -> Self {
        Self {
            archetypes: SlotMap::new(),
            component_id_map: AHashMap::new(),
            // component_entity_map: AHashMap::new()
        }
    }

    pub fn get_archetype(
        &mut self,
        entity: Entity,
    ) -> (
        &mut SlotMap<DefaultKey, Box<dyn ComponentInstance>>,
        &mut AHashMap<usize, DefaultKey>,
    ) {
        (
            self.archetypes.get_mut(entity.0).unwrap(),
            self.component_id_map.get_mut(&entity.0).unwrap(),
        )
    }

    pub fn get_archetype_iter(
        &mut self,
    ) -> (
        impl Iterator<
            Item = (
                DefaultKey,
                &mut SlotMap<DefaultKey, Box<dyn ComponentInstance>>,
            ),
        >,
        &mut AHashMap<DefaultKey, AHashMap<usize, DefaultKey>>,
    ) {
        let iter = (&mut self.archetypes).iter_mut();
        let id_map = &mut self.component_id_map;
        (iter, id_map)
    }

    pub fn all_entities(&self) -> Vec<Entity> {
        self.archetypes.keys().map(|x| Entity(x)).collect()
    }
}

unsafe impl Sync for ComponentStorage {}
unsafe impl Send for ComponentStorage {}
unsafe impl Sync for World {}
unsafe impl Send for World {}
