use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct Collider {
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl Collider {
    pub(crate) fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub(crate) fn get_size(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }

    pub(crate) fn get_half_size(&self) -> Vec2 {
        self.get_size() / 2.
    }
}
