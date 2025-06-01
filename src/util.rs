
use bevy::prelude::*;

//==============================================================================================
//        IsometricPosition
//==============================================================================================

pub struct IsometricPositionPlugin;

impl Plugin for IsometricPositionPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<IsometricPosition>()
            .add_systems(PreUpdate, update_isometric_positions)
        ;
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component, Default)]
#[require(Transform)]
pub struct IsometricPosition {
    pub x: f32,
    pub y: f32,
}

const I_HAT: Vec3 = Vec3::new(1.0, 0.0, 0.5);
const J_HAT: Vec3 = Vec3::new(-1.0, 0.0, 0.5);

fn update_isometric_positions(
    mut query: Query<(&mut Transform, &IsometricPosition), Changed<IsometricPosition>>,
) {
    for (mut transform, position) in query.iter_mut() {
        let pos = (position.x * I_HAT) + (position.y * J_HAT);
        let pos = Vec3::new(pos.x, transform.translation.y, pos.z);
        transform.translation = pos;
    }
}
