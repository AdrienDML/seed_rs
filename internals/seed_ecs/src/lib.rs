#![allow(dead_code, unused)]
use std::any::{TypeId, Any};
use std::alloc::Layout;


use entity::{Entities, Entity};

pub mod entity;
mod utils;

pub struct World {
    entities: Entities,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Entities::init(),
        }
    }
    
    pub fn spawn_entity(&mut self) -> &Entity {
        self.entities.spawn_entity()
    }

    pub fn enities(&self) -> &Entities {
        &self.entities
    }

}

