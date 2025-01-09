use super::{
    collision::AbsoluteCollider, collision::Collider, collision::StaticBody, trigger::TriggerLayer,
};
use bevy::{prelude::*, utils::hashbrown::HashMap};

#[derive(Debug, Clone, Copy)]
pub struct SpatialData<D> {
    pub entity: Entity,
    pub collider: AbsoluteCollider,
    pub data: D,
}

impl<D> SpatialData<D> {
    pub fn from_entity(
        entity: Entity,
        transform: &Transform,
        collider: &Collider,
        data: D,
    ) -> Self {
        Self {
            collider: collider.absolute(transform),
            entity,
            data,
        }
    }
}

#[derive(Debug, Component)]
pub struct SpatialHash<D> {
    cell_size: f32,
    objects: HashMap<(i32, i32), Vec<SpatialData<D>>>,
}

#[allow(dead_code)]
impl<D: Clone> SpatialHash<D> {
    pub fn new(cell_size: f32) -> Self {
        SpatialHash {
            cell_size,
            objects: HashMap::default(),
        }
    }

    pub fn new_with(cell_size: f32, data: impl IntoIterator<Item = SpatialData<D>>) -> Self {
        let mut slf = Self::new(cell_size);
        data.into_iter().for_each(|d| slf.insert(d));

        slf
    }

    fn hash(&self, position: &Vec2) -> (i32, i32) {
        (
            (position.x / self.cell_size).floor() as i32,
            (position.y / self.cell_size).floor() as i32,
        )
    }

    /// Inserts spatial data into map.
    ///
    /// Data will be added to all cells overlapped by data's [`AbsoluteCollider`].
    pub fn insert(&mut self, data: SpatialData<D>) {
        let (min_x, min_y) = self.hash(&Vec2::new(data.collider.min_x(), data.collider.min_y()));
        let (max_x, max_y) = self.hash(&Vec2::new(data.collider.max_x(), data.collider.max_y()));

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                self.objects.entry((x, y)).or_default().push(data.clone());
            }
        }
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn objects_in_cell_mut<'a>(
        &'a mut self,
        position: &Vec2,
    ) -> impl Iterator<Item = &'a mut SpatialData<D>> + 'a {
        let cell = self.hash(position);

        self.objects
            .get_mut(&(cell.0, cell.1))
            .into_iter()
            .flatten()
    }

    pub fn nearby_objects<'a>(
        &'a self,
        position: &Vec2,
    ) -> impl Iterator<Item = &'a SpatialData<D>> + 'a {
        let cell = self.hash(position);

        (-1..=1).flat_map(move |dx| {
            (-1..=1).flat_map(move |dy| {
                self.objects
                    .get(&(cell.0 + dx, cell.1 + dy))
                    .into_iter()
                    .flatten()
            })
        })
    }

    pub fn nearby_objects_mut<'a>(
        &'a mut self,
        position: &Vec2,
    ) -> impl Iterator<Item = &'a mut SpatialData<D>> + 'a {
        let cell = self.hash(position);

        let dx = cell.0 - 1..=cell.0 + 1;
        let dy = cell.1 - 1..=cell.1 + 1;

        self.objects
            .iter_mut()
            .filter_map(move |(k, v)| (dx.contains(&k.0) && dy.contains(&k.1)).then_some(v))
            .flatten()
    }
}

#[derive(Component)]
pub struct StaticBodyStorage;

pub type StaticBodyData = Option<TriggerLayer>;

// TODO: should there be a spatial hash for each level?
pub fn init_static_body_storage(mut commands: Commands) {
    commands.spawn((SpatialHash::<StaticBodyData>::new(32.), StaticBodyStorage));
}

pub fn store_static_body_in_spatial_map(
    map: Single<&mut SpatialHash<StaticBodyData>, With<StaticBodyStorage>>,
    static_body: Query<(Entity, &Transform, &Collider, Option<&TriggerLayer>), Added<StaticBody>>,
) {
    let mut map = map.into_inner();
    for (entity, transform, collider, trigger_layer) in static_body.iter() {
        map.insert(SpatialData {
            collider: collider.absolute(transform),
            data: trigger_layer.cloned(),
            entity,
        })
    }
}
