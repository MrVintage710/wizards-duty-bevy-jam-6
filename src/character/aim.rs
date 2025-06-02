use bevy::{prelude::*, text::cosmic_text::rustybuzz::script::SHARADA};

use crate::{arena::Ground, camera::MainCamera, GameState};

use super::{PlayerCharacter, ShootOrigin, AIM_HEIGHT};

//==============================================================================================
//        Aim Plugin
//==============================================================================================

pub struct AimPlugin;

impl Plugin for AimPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::InGame), setup_aim)
            .add_systems(PreUpdate, update_aim_target)
        ;
    }
}

//==============================================================================================
//        AimTarget
//==============================================================================================

#[derive(Component)]
#[require(Transform, Name::new("ShootTarget"))]
pub struct ShootTarget;

//==============================================================================================
//        Systems
//==============================================================================================

fn setup_aim(
    mut commands: Commands,
) {
    commands.spawn(ShootTarget);
}

fn update_aim_target(
    camera_query: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    ground: Single<&GlobalTransform, With<Ground>>,
    windows: Query<&Window>,
    mut aim_target: Single<&mut Transform, (With<ShootTarget>, Without<ShootOrigin>)>,
    shoot_origin : Single<&GlobalTransform, With<ShootOrigin>>,
    mut gizmos: Gizmos,
) {
    let Ok(windows) = windows.single() else {
        return;
    };

    let (camera, camera_transform) = *camera_query;

    let Some(cursor_position) = windows.cursor_position() else {
        return;
    };

    // Calculate a ray pointing from the camera into the world based on the cursor's position.
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Calculate if and where the ray is hitting the ground plane.
    let Some(distance) =
        ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up()))
    else {
        return;
    };
    let point = ray.get_point(distance);
    
    let player_translation_2d = Vec2::new(shoot_origin.translation().x, shoot_origin.translation().z);
    let point_location_2d = Vec2::new(point.x, point.z);
    let direction = (player_translation_2d - point_location_2d).normalize();
    let direction = direction * -3.0;
    let direction = Vec3::new(direction.x, 0.0, direction.y);
    aim_target.translation = shoot_origin.translation() + direction;

    gizmos.sphere(Isometry3d::from_translation(aim_target.translation), 0.2, Color::WHITE);
    gizmos.ray(shoot_origin.translation(), direction, Color::Srgba(Srgba::RED));
    
    // Draw a circle just above the ground plane at that position.
    gizmos.circle(
        Isometry3d::new(
            point + ground.up() * 0.01,
            Quat::from_rotation_arc(Vec3::Z, ground.up().as_vec3()),
        ),
        0.2,
        Color::WHITE,
    ).resolution(8);
}

