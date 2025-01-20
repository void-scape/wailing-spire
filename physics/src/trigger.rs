use std::marker::PhantomData;

use super::{
    collision::Collider,
    layers::TriggersWith,
    prelude::CollidesWith,
    spatial::{SpatialData, SpatialHash},
};
use bevy::{prelude::*, utils::hashbrown::HashMap};

/// Marks an entity as a [`TriggerEvent`] source.
///
/// Can exist in combination with a [`StaticBody`] or [`DynamicBody`].
///
/// Will trigger with any [`TriggersWith`] layer present in the entity.
#[derive(Debug, Default, Clone, Copy, Component)]
pub struct Trigger(pub Collider);

/// A list of all entities within a trigger.
#[derive(Component)]
pub struct Triggers<T>(smallvec::SmallVec<[Entity; 4]>, PhantomData<T>);

impl<T> Default for Triggers<T> {
    fn default() -> Self {
        Self(smallvec::SmallVec::default(), PhantomData)
    }
}

impl<T> Triggers<T> {
    pub fn entities(&self) -> &[Entity] {
        &self.0
    }

    fn clear(&mut self) {
        self.0.clear();
    }
}

/// In the case of trigger triggers trigger, two triggers will each trigger, both triggering the
/// other trigger.
#[derive(Debug, Clone, Copy, Event)]
pub struct TriggerEvent {
    pub trigger: Entity,
    pub target: Entity,
}

/// Fires when a trigger enters the target entity.
///
/// Uses the [`TriggerEvent`] internally.
#[derive(Debug, Clone, Copy, Event)]
pub struct TriggerEnter {
    pub trigger: Entity,
    pub target: Entity,
}

/// Fires when a trigger exits the target entity.
///
/// Uses the [`TriggerEvent`] internally.
#[derive(Debug, Clone, Copy, Event)]
pub struct TriggerExit {
    pub trigger: Entity,
    pub target: Entity,
}

pub fn emit_trigger_states(
    mut enter: EventWriter<TriggerEnter>,
    mut exit: EventWriter<TriggerExit>,
    mut reader: EventReader<TriggerEvent>,
    mut active: Local<HashMap<Entity, smallvec::SmallVec<[Entity; 4]>>>,
) {
    let events = reader.read().collect::<Vec<_>>();

    for event in events.iter() {
        let entry = active.entry(event.trigger).or_default();
        if !entry.contains(&event.target) {
            enter.send(TriggerEnter {
                trigger: event.trigger,
                target: event.target,
            });
            entry.push(event.target);
        }
    }

    active.retain(|entity, targets| {
        if events
            .iter()
            .any(|ev| ev.trigger == *entity && targets.contains(&ev.target))
        {
            true
        } else {
            for target in targets.iter() {
                exit.send(TriggerExit {
                    trigger: *entity,
                    target: *target,
                });
            }

            false
        }
    });
}

pub fn handle_triggers<T: Component>(
    triggers: Query<(Entity, &GlobalTransform, &Trigger), With<T>>,
    bodies: Query<(Entity, &GlobalTransform, &Collider, &TriggersWith<T>)>,
    mut body_triggers: Query<&mut Triggers<T>>,
    mut writer: EventWriter<TriggerEvent>,
) {
    for mut trigger in body_triggers.iter_mut() {
        trigger.clear();
    }

    let dynamic_body_map = SpatialHash::new_with(
        64.,
        bodies
            .iter()
            .map(|(e, t, c, _)| SpatialData::from_entity(e, t, c, ())),
    );

    for (entity, transform, trigger) in triggers.iter() {
        let collider = trigger.0.global_absolute(transform);

        for SpatialData {
            entity: e,
            collider: c,
            ..
        } in dynamic_body_map.nearby_objects(&collider.position())
        {
            if *e != entity && collider.collides_with(c) {
                writer.send(TriggerEvent {
                    trigger: entity,
                    target: *e,
                });

                if let Ok(mut triggers) = body_triggers.get_mut(*e) {
                    triggers.0.push(entity);
                }
            }
        }
    }
}
