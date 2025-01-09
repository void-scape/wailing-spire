use bevy::{prelude::*, utils::HashMap};
use std::{hash::Hash, marker::PhantomData, time::Duration};

pub struct AnimationPlugin<A> {
    _marker: PhantomData<A>,
}

impl<A: Animation> Default for AnimationPlugin<A> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<A: Animation> Plugin for AnimationPlugin<A> {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animation::<A>);
    }
}

pub trait Animation: Clone + Hash + PartialEq + Eq + Send + Sync + 'static {}

impl<T> Animation for T where T: Clone + Hash + PartialEq + Eq + Send + Sync + 'static {}

#[derive(Debug, Component)]
pub struct AnimationController<A> {
    index_map: HashMap<A, (usize, usize)>,
    active_index: Option<(A, (usize, usize), usize)>,
    timer: Timer,
}

impl<A> Default for AnimationController<A> {
    fn default() -> Self {
        Self {
            index_map: HashMap::default(),
            active_index: None,
            timer: Timer::new(Duration::from_secs_f32(1.0 / 2.0), TimerMode::Repeating),
        }
    }
}

#[allow(dead_code)]
impl<A: Animation> AnimationController<A> {
    pub fn new(speed: f32, map: impl std::iter::IntoIterator<Item = (A, (usize, usize))>) -> Self {
        let mut index_map = HashMap::default();
        for (dir, range) in map.into_iter() {
            index_map.insert(dir, range);
        }

        Self {
            index_map,
            active_index: None,
            timer: Timer::new(Duration::from_secs_f32(1.0 / speed), TimerMode::Repeating),
        }
    }

    pub fn index(&self) -> Option<usize> {
        self.active_index.as_ref().map(|(_, _, i)| *i)
    }

    pub fn set_animation(&mut self, animation: A) {
        if let Some(range) = self.get_range(&animation) {
            self.active_index = Some((animation.clone(), range, range.0));
            self.timer.reset();
        }
    }

    pub fn clear(&mut self) {
        self.active_index = None;
    }

    pub fn active_animation(&self) -> Option<&A> {
        self.active_index.as_ref().map(|(dir, _, _)| dir)
    }

    fn update(&mut self, time: &Time) {
        self.timer.tick(time.delta());

        if let Some((_, (start, end), index)) = &mut self.active_index {
            if self.timer.just_finished() {
                *index += 1;
                if *index >= *end {
                    *index = *start;
                }
            }
        }
    }

    fn get_range(&self, animation: &A) -> Option<(usize, usize)> {
        self.index_map.get(animation).cloned()
    }
}

fn animation<A: Animation>(
    mut query: Query<(&mut Sprite, &mut AnimationController<A>)>,
    time: Res<Time>,
) {
    for (mut sprite, mut animation) in query.iter_mut() {
        animation.update(&time);
        if let Some(index) = animation.index() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = index;
            }
        }
    }
}
