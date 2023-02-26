use bevy::prelude::*;

/// Despawn all entities with a given component type
pub(crate) fn despawn_with<T: Component>(
    mut commands: Commands,
    entity_query: Query<Entity, With<T>>,
) {
    for target_entity in &entity_query {
        commands.entity(target_entity).despawn_recursive();
    }
}

pub(crate) fn cursor_position_in_world(
    windows: &Windows,
    cursor_position: Vec2,
    camera_transform: &GlobalTransform,
    camera: &Camera,
) -> Vec3 {
    let window = windows.primary();

    let window_size = Vec2::new(window.width(), window.height());

    // Convert screen position [0..resolution] to ndc [-1..1]
    // (ndc = normalized device coordinates)
    let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();
    let ndc = (cursor_position / window_size) * 2.0 - Vec2::ONE;
    ndc_to_world.project_point3(ndc.extend(0.0))
}
