mod knight {
    use ::bevy::prelude::ReflectComponent;
    #[derive(::bevy::ecs::component::Component, ::bevy::reflect::Reflect)]
    #[reflect(Component)]
    pub struct Knight;
}
#[allow(unused)]
pub use knight::Knight;
