use std::f64::consts::PI;

use avian3d::prelude::*;
use bevy::{animation::AnimationTarget, prelude::*};
use bevy_seedling::sample::SamplePlayer;
use bevy_tnua::{prelude::*, TnuaNotPlatform};
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use crate::{arena::beacon::BeaconQuery, assets::{EnemyAnimationGraphs, EnemyAssets, WizardAssets}, character::PlayerCharacter, enemy::{Enemy, EnemySpawnAnimationComplete, EnemyType, SpawnEnemy, SpecialEnemyBehavior}, util::{AnimatedModelFor, AnimatedSceneCreated, GameCollisionLayer, Health, SceneRootWithAnimation}};

use super::EnemyBehavior;

const MINION_HEIGHT: f32 = 1.0;
const MINION_HEALTH: u32 = 5;
const MINION_SPEED: f32 = 3.0;
const MINION_AGRO_RANGE: f32 = 5.0;
const MINION_ATTACK_COOLDOWN: f32 = 2.0;
const MINION_ATTACK_RANGE: f32 = 1.5;
const MINION_BEACON_ATTACK_RANGE: f32 = 2.5;

//==============================================================================================
//        Minion Plugin
//==============================================================================================

pub struct MinionPlugin;

impl Plugin for MinionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(spawn_minion_enemy)
            .add_systems(Update, (minion_goto, minion_attack_player, minion_idle, manage_minion_animation).chain().in_set(SpecialEnemyBehavior))
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
//       Minion Events 
//==============================================================================================

#[derive(Event, Clone)]
pub struct MinionStabbed;

pub fn minion_stabbed(
    events: Trigger<MinionStabbed>,
    mut query: Query<&mut Minion>,
) {
    println!("Stabbed")
}

//==============================================================================================
//        Spawn a minion Enemy
//==============================================================================================

pub fn spawn_minion_enemy(
    trigger : Trigger<SpawnEnemy>,
    mut commands : Commands,
    enemy_assets : Res<EnemyAssets>,
    enemy_animation_graphs : Res<EnemyAnimationGraphs>
) {
    if trigger.1 != EnemyType::Minion { return }
    let position = trigger.0;
    
    commands.spawn((
        Name::new("Minion"),
        Transform::from_translation(Vec3::new(position.x, MINION_HEIGHT, position.z)),
        Minion::default(),
        Enemy {
            height_from_ground: MINION_HEIGHT,
            speed: MINION_SPEED,
        },
        Health::new(MINION_HEALTH),
        EnemyBehavior::Spawning,
        CollisionLayers::new(GameCollisionLayer::Enemy, [
            GameCollisionLayer::Player, 
            GameCollisionLayer::Enemy, 
            GameCollisionLayer::Default, 
            GameCollisionLayer::Spell,
        ]),
        RigidBody::Dynamic,
        Collider::capsule(0.5, 0.5),
        TnuaController::default(),
        TnuaAvian3dSensorShape(Collider::cylinder(0.49, 0.0)),
        TnuaNotPlatform,
        SceneRootWithAnimation::new(enemy_assets.skeleton_minion.clone())
            .with_animation_graph(enemy_animation_graphs.minion_graph.clone())
            .with_animation(enemy_animation_graphs.minion_spawn)
            .with_transform(Transform::from_translation((0.0, -1.0, 0.0).into()).with_rotation(Quat::from_rotation_y(PI as f32))),
    )).observe(on_minion_scene_added);
}

pub fn on_minion_scene_added(
    trigger : Trigger<AnimatedSceneCreated>,
    named : Query<&Name>,
    mut commands : Commands,
    spawner : Res<SceneSpawner>,
    rig : Query<(Entity, &Name), With<AnimationTarget>>,
    assets : Res<EnemyAssets>
) {
    let controler = trigger.target().clone();
    println!("{:?}", named.get(controler));
    commands.entity(trigger.0).observe(move |_ : Trigger<EnemySpawnAnimationComplete>, mut enemies : Query<&mut EnemyBehavior, With<Enemy>>| {
        let Ok(mut enemy_behavior) = enemies.get_mut(controler) else { return; };
        *enemy_behavior = EnemyBehavior::Idle;
    });
    
    let hand = spawner.iter_instance_entities(trigger.1)
        .filter_map(|e| rig.get(e).map(|part| Some(part)).unwrap_or(None))
        .find(|part| part.1.as_str() == "handslot.r")
    ;
    
    if let Some(hand) = hand {
        let transform = Transform::from_rotation(Quat::from_rotation_y(PI as f32));
        commands.entity(hand.0).insert(
            SceneRootWithAnimation::new(assets.skeleton_blade.clone())
                .with_transform(transform)
        );
    }
}

//==============================================================================================
//        Animating the Minion
//==============================================================================================

fn manage_minion_animation(
    minions : Query<(&LinearVelocity, &EnemyBehavior), With<Minion>>,
    mut minion_animated_models : Query<(&AnimatedModelFor, &mut AnimationPlayer)>,
    animations : Res<EnemyAnimationGraphs>,
) {
    for (animated_model_for, mut animation_player) in minion_animated_models.iter_mut() {
        let Ok((minion_velocity, minion_behavior)) = minions.get(animated_model_for.0) else { continue; };
        if matches!(minion_behavior, EnemyBehavior::Spawning) { continue; }
        
        let velocity_magnitude = minion_velocity.length();
        let is_stab_finished = animation_player.animation(animations.minion_stab).map(|animation| animation.is_finished()).unwrap_or(false);
        if is_stab_finished {
            animation_player.stop(animations.minion_stab);
        }
        
        if velocity_magnitude >= 0.05 {
            animation_player.play(animations.minion_run_bottom).repeat();
            
            animation_player.stop(animations.minion_idle);
            if !animation_player.is_playing_animation(animations.minion_stab) {
                animation_player.play(animations.minion_run_top).repeat();
            }
        } else {
            animation_player.stop(animations.minion_run_bottom);
            animation_player.stop(animations.minion_run_top);
            animation_player.play(animations.minion_idle).repeat();
        }
    }
}


//==============================================================================================
//        Minion Behavior
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

//==============================================================================================
//        Minion Attack Player
//==============================================================================================

pub fn minion_attack_player(
    mut commmands : Commands,
    player : Single<&Transform, With<PlayerCharacter>>,
    player_assets : Res<WizardAssets>,
    mut minions : Query<(Entity, &mut EnemyBehavior, &mut TnuaController, &Transform, &mut Minion)>,
    mut minion_animators : Query<(&AnimatedModelFor, &mut AnimationPlayer)>,
    enemy_animation_graphs : Res<EnemyAnimationGraphs>,
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
            *behavior = EnemyBehavior::Idle;
        }
        
        minion.attack_cooldown.tick(time.delta());
        
        if enemies_within_attack_range.contains(&entity) && minion.attack_cooldown.just_finished() {
            let Some((_, mut animation_player)) = minion_animators.iter_mut().find(|i| i.0.0 == entity) else { continue };
            // if animation_player.is_playing_animation(enemy_animation_graphs.minion_run_top) {
                animation_player.stop(enemy_animation_graphs.minion_run_top);
            // }
            animation_player.start(enemy_animation_graphs.minion_stab);
            commmands.spawn(
                SamplePlayer::new(player_assets.oof1.clone())
            );
        }
        
    }
}

//==============================================================================================
//        Minion Idle Behavior
//==============================================================================================

pub fn minion_idle(
    mut enemy : Query<(Entity,&mut TnuaController, &mut EnemyBehavior, &Enemy, &LinearVelocity, &Transform)>,
    player : Single<&Transform, With<PlayerCharacter>>,
    spacial_query : SpatialQuery,
    beacon : BeaconQuery,
) {
    for (entity, mut controller, mut behavior, enemy, rb, transform) in enemy.iter_mut() {
        if !(matches!(behavior.as_ref(), &EnemyBehavior::Idle)) { return }
        controller.basis(TnuaBuiltinWalk {
            desired_velocity: (0.0, 0.0, 0.0).into(),
            float_height: enemy.height_from_ground,
            ..default()
        });
        
        let enemies_within_agro_range = spacial_query.shape_intersections(
            &Collider::sphere(MINION_AGRO_RANGE),
            player.translation, 
            Quat::default(),                 // Shape rotation
            &SpatialQueryFilter::default()
        );
        
        if enemies_within_agro_range.contains(&entity) {
            *behavior = EnemyBehavior::AttackPlayer;
        } else if beacon.within_range(transform, MINION_BEACON_ATTACK_RANGE) {
            *behavior = EnemyBehavior::AttackBeacon;
        } else {
            *behavior = EnemyBehavior::goto(beacon.closest_point(transform, MINION_BEACON_ATTACK_RANGE))
        }
    }
}
