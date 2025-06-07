use avian3d::prelude::{Collider, SpatialQuery, SpatialQueryFilter};
use bevy::{color::palettes, ecs::entity, gizmos, platform::collections::HashMap, prelude::*};
use bevy_tnua::prelude::*;
use rand::Rng;
use strum::{EnumCount, FromRepr};
use vleue_navigator::{prelude::*, Path};
use weighted_rand::{builder::{NewBuilder, WalkerTableBuilder}, table::WalkerTable};

use crate::{arena::NavmeshQuery, enemy::minion::MinionPlugin, util::{vec2_vec3, Health}};

pub mod minion;

const MAX_ENEMIES: u32 = 1000;
const SPAWN_RADIUS: i32 = 7;
const MAX_ENEMIES_PER_SPAWN: u32 = 20;
const ENEMY_CROWDING_SPACE : f32 = 5.0;
const ENEMY_SEPORATION_FACTOR : f32 = 0.9;
const ENENY_CORNER_CUTTING : f32 = 0.5;

//==============================================================================================
//        Enemy Plguin
//==============================================================================================

pub struct EnemyPlugin(bool);

impl Default for EnemyPlugin {
    fn default() -> Self {
        EnemyPlugin(false)
    }
}

impl EnemyPlugin {
    pub fn new(debug: bool) -> Self {
        EnemyPlugin(debug)
    }
}

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(MinionPlugin)
            
            .init_resource::<EnemyCount>()
            
            .add_observer(spawn_enemies)
        
            .add_systems(Update, (enemy_idle_and_spawning, enemy_goto).chain().in_set(DefaultEnemyBehavior))
            .add_systems(PostUpdate, check_for_dead_enemies)
        ;
        
        if self.0 {
            app.add_systems(Update, debug_goto.in_set(DefaultEnemyBehavior));
        }
    }
}

//==============================================================================================
//        Enemy Component
//==============================================================================================

#[derive(Component)]
pub struct Enemy {
    pub height_from_ground : f32,
    pub speed : f32,
}

//============================================================================================== 
//        Enemy General Stuff
//==============================================================================================

#[derive(Resource, Clone, Debug, Default)]
pub struct EnemyCount(u32);

#[repr(usize)]
#[derive(FromRepr, Hash, PartialEq, Eq, PartialOrd, Ord, EnumCount)]
pub enum EnemyType {
    Minion,
    Mage,
}

//==============================================================================================
//        Enemy Behaviors
//==============================================================================================

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefaultEnemyBehavior;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpecialEnemyBehavior;

#[derive(Component, Default, Debug)]
pub enum EnemyBehavior {
    #[default]
    Idle,
    Spawning,
    Guard,
    Goto(Vec2, Option<Path>, usize),
    AttackBeacon,
    AttackPlayer,
}

impl EnemyBehavior {    
    pub fn goto(position : Vec2) -> Self {
        EnemyBehavior::Goto(position, None, 0)
    }
    
    pub fn is_goto(&self) -> bool {
        matches!(self, EnemyBehavior::Goto(..))
    }
    
    pub fn is_attack_player(&self) -> bool {
        matches!(self, EnemyBehavior::AttackPlayer)
    }
}

//==============================================================================================
//         Spawn Enemy Event
//==============================================================================================

#[derive(Event, Clone)]
pub struct EnemySpawnAnimationComplete;

#[derive(Event)]
pub struct SpawnEnemiesEvent(Vec3, u32, WalkerTable);

impl SpawnEnemiesEvent {
    pub fn collect(&self) -> Vec<EnemyType> {
        (0..self.1).filter_map(|_| EnemyType::from_repr(self.2.next()) ).collect()
    }
}

pub struct SpawnEnemiesEventBuilder {
    position: Vec3,
    number_of_enemies: u32,
    weights : [u32; EnemyType::COUNT]
}

impl SpawnEnemiesEventBuilder {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            number_of_enemies : 1,
            weights : [0; EnemyType::COUNT]
        }
    }
    
    pub fn with_number_of_enemies(mut self, number_of_enemies: u32) -> Self {
        self.number_of_enemies = number_of_enemies;
        self
    }
    
    pub fn with_weight(mut self, enemy_type : EnemyType, weight : u32) -> Self {
        self.weights[enemy_type as usize] = weight;
        self
    }
    
    pub fn build(self) -> SpawnEnemiesEvent {
        SpawnEnemiesEvent(self.position, self.number_of_enemies, WalkerTableBuilder::new(&self.weights).build())
    }
}

#[derive(Event)]
pub struct SpawnEnemy(Vec3, EnemyType);

pub fn spawn_enemies(
    trigger : Trigger<SpawnEnemiesEvent>,
    mut commands : Commands,
    mut enemy_count: ResMut<EnemyCount>
) {
    let mut rng = rand::rng();
    for enemy in trigger.collect().into_iter() {
        let rand_vec = Vec3::new(rng.random_range(-SPAWN_RADIUS..=SPAWN_RADIUS) as f32, 0.0, rng.random_range(-SPAWN_RADIUS..=SPAWN_RADIUS) as f32);
        if enemy_count.0 > MAX_ENEMIES { return }
        commands.trigger(SpawnEnemy(trigger.0 + rand_vec, enemy));
        enemy_count.0 += 1;
    }
}

//==============================================================================================
//        Enemy Systems
//==============================================================================================

pub fn check_for_dead_enemies (
    mut commands : Commands,
    enemies : Query<(Entity, &Health), With<Enemy>>,
    mut enemy_count : ResMut<EnemyCount>
) {
    for (entity, enemy) in enemies.iter() {
        if enemy.current_health == 0 {
            enemy_count.0 -= 1;
            commands.entity(entity).despawn();
        }
    }
}

//==============================================================================================
//        Enemy Goto
//==============================================================================================

pub fn enemy_goto(
    mut enemies : Query<(Entity, &Transform, &mut TnuaController, &mut EnemyBehavior, &Enemy)>,
    navmesh : NavmeshQuery,
    spacial_query: SpatialQuery,
    mut gizmos : Gizmos,
) {
    //Get all enemies that are in the goto behavior
    let mut enemies = enemies.iter_mut()
        .filter_map(|(entity, transform, controller, behavior, enemy)| {
            if !behavior.is_goto() { return None };
            Some((entity, (transform, controller, behavior, enemy)))
        })
        .collect::<HashMap<_, _>>()
    ;

    let intended_velocities = enemies.iter().map(|(entity, (transform, _, behavior, enemy))| {
        let nearby_enmemies = spacial_query.shape_intersections(
            &Collider::sphere(2.0), 
            transform.translation, 
            Quat::default(), 
            &SpatialQueryFilter::default(), 
        );

        let nearby_enemies =  nearby_enmemies.iter().filter_map(|e| {
            if e == entity { return None }
            enemies.get(e).map(|i| Some(i.0)).unwrap_or(None)
        });

        let mut avoiding = 0;
        let mut speration_velocity = Vec2::default();
        let current_location = transform.translation.xz();
        
        for other in nearby_enemies {
            let other_location = other.translation.xz();
            let distance = current_location.distance(other_location);
            if distance < ENEMY_CROWDING_SPACE {
                let direction_away = (current_location - other_location).normalize_or_zero();
                let weighted_velocity = direction_away / distance;
                speration_velocity += weighted_velocity;
                avoiding += 1;
            }
        }

        if avoiding > 0 {
            speration_velocity /= avoiding as f32;
            speration_velocity *= ENEMY_SEPORATION_FACTOR;
        }

        let EnemyBehavior::Goto(destination, Some(path), index) = behavior.as_ref() else { return (entity.clone(), (speration_velocity, speration_velocity.normalize_or_zero()))};
        let current_node = path.path.get(*index).unwrap_or(destination);

        let desired_velocity = (current_node - current_location).normalize_or_zero() * enemy.speed;
        
        let indended_velocity = speration_velocity + desired_velocity;
        gizmos.ray(transform.translation + enemy.height_from_ground, vec2_vec3(speration_velocity), bevy::color::palettes::tailwind::BLUE_500);
        gizmos.ray(transform.translation + enemy.height_from_ground, vec2_vec3(desired_velocity), bevy::color::palettes::tailwind::RED_500);
        gizmos.ray(transform.translation + enemy.height_from_ground, vec2_vec3(indended_velocity), bevy::color::palettes::tailwind::VIOLET_500);
        (entity.clone(), (indended_velocity, indended_velocity.normalize_or_zero()))
    }).collect::<HashMap<Entity, _>>();
    
    for (entity, (intended_velocity, intended_direction)) in intended_velocities.into_iter() {
        let Some((transform, controller, behavior, enemy)) = enemies.get_mut(&entity) else { unreachable!() };
        let direction = if intended_direction.length_squared() == 0.0 {None} else {Some(Dir3::from_xyz_unchecked(intended_direction.x, 0.0, intended_direction.y))};
        controller.basis(TnuaBuiltinWalk {
            desired_velocity: (intended_velocity.x, 0.0, intended_velocity.y).into(),
            float_height: enemy.height_from_ground,
            desired_forward: direction,
            ..default()
        });

        let should_be_idle = {
            let EnemyBehavior::Goto(destination, path, index) = behavior.as_mut() else { continue };
            if path.is_none() {
                *path = navmesh.path_from_tranform(&transform, *destination);
                if path.is_none() { continue; }
            }
            let path = path.as_ref().unwrap();
            let current_node = path.path.get(*index).unwrap_or(destination);
    
            if transform.translation.xz().distance(*current_node) < ENENY_CORNER_CUTTING {
                *index += 1
            }
    
            *index >= path.path.len()
        };

        if should_be_idle { **behavior = EnemyBehavior::Idle }
    }
}

pub fn debug_goto (
    enemy_behaviors : Query<(&EnemyBehavior, &Transform)>,
    mut gizmos : Gizmos
) {
    for (enemy_behavior, transform) in enemy_behaviors.iter() {
        if let EnemyBehavior::Goto(destination, Some(path), index) = enemy_behavior {
            let mut points : Vec<Vec3> = vec![(transform.translation.x, 0.1, transform.translation.z).into()];
            path.path.iter().skip(*index).for_each(|point| points.push(Vec3::new(point.x, 0.1, point.y)));
            if let Some(point) = points.get(*index + 1) {
                gizmos.sphere(Isometry3d::from_translation(*point), 0.3, bevy::color::palettes::tailwind::EMERALD_600);
            } else {
                gizmos.sphere(Isometry3d::from_translation(Vec3::new(destination.x, 0.1, destination.y)), 0.3, bevy::color::palettes::tailwind::EMERALD_600);
            }
            gizmos.linestrip(points, bevy::color::palettes::tailwind::EMERALD_600);
        }
    }
}

//==============================================================================================
//        Enemy Idle
//==============================================================================================

pub fn enemy_idle_and_spawning(
    mut enemy : Query<(&mut TnuaController, &EnemyBehavior, &Enemy)>,
) {
    for (mut controller, behavior, enemy) in enemy.iter_mut() {
        if !(matches!(behavior, &EnemyBehavior::Idle | &EnemyBehavior::Spawning)) {return;}
        controller.basis(TnuaBuiltinWalk {
            desired_velocity: (0.0, 0.0, 0.0).into(),
            float_height: enemy.height_from_ground,
            ..default()
        });
    }
}

//==============================================================================================
//        Enemy General
//==============================================================================================

#[derive(Component)]
#[relationship(relationship_target = Hands)]
pub struct HandFor(pub Entity);

/// This is the "relationship target" component.
/// It will be automatically inserted and updated to contain
/// all entities that currently "like" this entity.
#[derive(Component, Deref)]
#[relationship_target(relationship = HandFor)]
pub struct Hands(Vec<Entity>);