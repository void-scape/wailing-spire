use crate::spire::LevelExt;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct LevelRegistry(Vec<Box<dyn LevelExt>>);

impl LevelRegistry {
    pub fn push<T: LevelExt>(&mut self, level: T) {
        self.0.push(Box::new(level));
    }

    pub fn levels(&self) -> &[Box<dyn LevelExt>] {
        &self.0
    }
}

