use std::f64::consts::PI;

use avian3d::prelude::*;
use bevy::{prelude::*, transform};
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use crate::{assets::EnemyAssets, character::PlayerCharacter, enemy::{minion, Enemy, EnemyType, SpawnEnemy, SpecialEnemyBehavior}, util::GameCollisionLayer};

use super::EnemyBehavior;

const MINION_HEIGHT: f32 = 1.0;
const MINION_HEALTH: i32 = 5;
const MINION_SPEED: f32 = 3.0;
const MINION_AGRO_RANGE: f32 = 10.0;
const MINION_ATTACK_COOLDOWN: f32 = 2.0;
const MINION_ATTACK_RANGE: f32 = 1.5;

//==============================================================================================
//        Minion Plugin
//==============================================================================================

pub struct MinionPlugin;

impl Plugin for MinionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(spawn_minion_enemy)
            .add_systems(Update, (minion_goto, minion_attack_player).in_set(SpecialEnemyBehavior))
        ;
    }
}

//==============================================================================================
//        Minion Component
//==============================================================================================

#[derive(Component)]
pub struct Minion {
    pub attack_cooldown: Timer,
}

impl Default for Minion {
    fn default() -> Self {
        Self {
            attack_cooldown: Timer::from_seconds(MINION_ATTACK_COOLDOWN, TimerMode::Repeating),
        }
    }
}


//==============================================================================================
//        Spawn a minion Enemy
//==============================================================================================

pub fn spawn_minion_enemy(
    trigger : Trigger<SpawnEnemy>,
    mut commands : Commands,
    enemy_assets : Res<EnemyAssets>
) {
    if trigger.1 != EnemyType::Minion { return }
    let position = trigger.0;
    
    commands.spawn((
        Transform::from_translation(Vec3::new(position.x, MINION_HEIGHT, position.z)),
        Minion::default(),
        Enemy {
            health: MINION_HEALTH,
            height_from_ground: MINION_HEIGHT,
            speed: MINION_SPEED,
        },
        EnemyBehavior::goto((-2.5, -2.5).into()),
        CollisionLayers::new(GameCollisionLayer::Enemy, [GameCollisionLayer::Player, GameCollisionLayer::Default, GameCollisionLayer::Spell]),
        RigidBody::Dynamic,
        Collider::capsule(0.5, 0.5),
        TnuaController::default(),
        TnuaAvian3dSensorShape(Collider::cylinder(0.49, 0.0)),
        children![
            (
                Transform::from_translation((0.0, -1.0, 0.0).into()).with_rotation(Quat::from_rotation_y(PI as f32)),
                SceneRoot(enemy_assets.skeleton_minion.clone())
            )
        ]
    ));
    // Implementation details here
}

//==============================================================================================
//        Minion Logic
//==============================================================================================


pub fn minion_goto (
    player : Single<&Transform, With<PlayerCharacter>>,
    mut minions : Query<&mut EnemyBehavior, With<Minion>>,
    spacial_query: SpatialQuery,
) {
    let entities = spacial_query.shape_intersections(
        &Collider::sphere(MINION_AGRO_RANGE),
        player.translation, 
        Quat::default(),                 // Shape rotation
        &SpatialQueryFilter::default()
    );
    
    for entity in entities.iter() {
        let Ok(mut behavior) = minions.get_mut(*entity) else { continue; };
        if !behavior.is_goto() {continue;}
        *behavior = EnemyBehavior::AttackPlayer;
    }
}

pub fn minion_attack_player(
     player : Single<&Transform, With<PlayerCharacter>>,
     mut minions : Query<(Entity, &mut EnemyBehavior, &mut TnuaController, &Transform, &mut Minion)>,
     spacial_query : SpatialQuery,
     time : Res<Time>
) {
    let enemies_within_agro_range = spacial_query.shape_intersections(
        &Collider::sphere(MINION_AGRO_RANGE),
        player.translation, 
        Quat::default(),                 // Shape rotation
        &SpatialQueryFilter::default()
    );
    
    let enemies_within_attack_range = spacial_query.shape_intersections(
        &Collider::sphere(MINION_ATTACK_RANGE),
        player.translation, 
        Quat::default(),                 // Shape rotation
        &SpatialQueryFilter::default()
    );
    
    for (entity, mut behavior, mut controller, transform, mut minion) in minions.iter_mut() {
        if !behavior.is_attack_player() {continue;}
        
        let direction_to_player = (player.translation.xz() - transform.translation.xz()).normalize_or_zero();
        let move_vector = direction_to_player * MINION_SPEED;
        
        controller.basis(TnuaBuiltinWalk {
            desired_velocity: (move_vector.x, 0.0, move_vector.y).into(),
            float_height: MINION_HEIGHT,
            desired_forward: Some(Dir3::from_xyz_unchecked(direction_to_player.x, 0.0, direction_to_player.y)),
            ..default()
        });
        
        if !enemies_within_agro_range.contains(&entity) {
            *behavior = EnemyBehavior::None;
        }
        
        if enemies_within_attack_range.contains(&entity) && minion.attack_cooldown.just_finished() {
            println!("ATACK!");
        }
        
        minion.attack_cooldown.tick(time.delta());
    }
}
