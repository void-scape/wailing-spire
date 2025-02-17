use crate::spire::QueryLevelFields;
use bevy_ldtk_scene::prelude::LevelMetaExt;

pub trait Level: LevelMetaExt + QueryLevelFields {}

impl<T> Level for T where T: LevelMetaExt + QueryLevelFields {}
