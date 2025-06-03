use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use vleue_navigator::{prelude::*, Path};
use crate::{assets::EnemyAssets, enemy::Enemy, util::GameCollisionLayer};

use super::EnemyBehavior;

const MINION_HEIGHT: f32 = 1.0;
const MINION_HEALTH: i32 = 5;

//==============================================================================================
//        Spawn a minion Enemy
//==============================================================================================

pub fn spawn_minion_enemy(commands : &mut Commands, position : Vec3, enemy_assets : &EnemyAssets) {
    
    commands.spawn((
        Transform::from_translation(Vec3::new(position.x, MINION_HEIGHT, position.z)),
        Minion,
        Enemy {
            health: MINION_HEALTH,
        },
        EnemyBehavior::attack_beacon(),
        CollisionLayers::new(GameCollisionLayer::Enemy, [GameCollisionLayer::Player, GameCollisionLayer::Default, GameCollisionLayer::Spell]),
        RigidBody::Dynamic,
        Collider::capsule(0.5, 0.5),
        TnuaController::default(),
        TnuaAvian3dSensorShape(Collider::cylinder(0.49, 0.0)),
        LockedAxes::ROTATION_LOCKED,
        children![
            (
                Transform::from_translation((0.0, -1.0, 0.0).into()),
                SceneRoot(enemy_assets.skeleton_minion.clone())
            )
        ]
    ));
    // Implementation details here
}

//==============================================================================================
//        Minion Logic
//==============================================================================================

#[derive(Component)]
pub struct Minion;

pub fn minion_behavior(
    mut minion : Query<(&mut EnemyBehavior, &mut TnuaController, &Transform), With<Minion>>,
    ground : Single<(&ManagedNavMesh, &NavMeshStatus)>,
    nav_meshes : Res<Assets<NavMesh>>
) {
    if *ground.1 != NavMeshStatus::Built { return }
    let Some(navmesh) = nav_meshes.get(ground.0.id()) else { return };
    
    for (mut enemy_behavior, mut controller, transform) in minion.iter_mut() {
        match enemy_behavior.as_mut() {
            EnemyBehavior::None => return,
            EnemyBehavior::Guard => todo!(),
            EnemyBehavior::AttackBeacon(path, index) => {
                while_attacking_beacon(controller.as_mut(), path, index, navmesh, transform);
            },
            EnemyBehavior::AttackPlayer => todo!(),
        }
    }
}

fn while_attacking_beacon(
    controller : &mut TnuaController,
    current_path : &mut Option<Path>,
    index : &mut usize,
    navmesh : &NavMesh,
    transform : &Transform
) {
    let current_location = Vec2::new(transform.translation.x, transform.translation.z);
    if let None = current_path {
        let path = navmesh.path(current_location, (0.0, 0.0).into());
        *current_path = path;
    }
    
    let current_path = current_path.as_ref().unwrap();
    let Some(mut current_node) = current_path.path.get(*index) else { return };
    
    if current_location.distance(*current_node) < 0.3 {
        *index += 1;
        if let Some(next_node) = current_path.path.get(*index) {
            current_node = next_node;
        }
    }
    
    let direction_to_next = (current_node - current_location).normalize_or_zero();
    let move_2d = direction_to_next * 4.0;
    controller.basis(TnuaBuiltinWalk {
        desired_velocity: (move_2d.x, 0.0, move_2d.y).into(),
        float_height: MINION_HEIGHT,
        ..default()
    });
}
