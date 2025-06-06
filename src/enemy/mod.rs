use avian3d::position;
use bevy::{ecs::system::IntoObserverSystem, prelude::*};
use bevy_tnua::prelude::*;
use minion::{spawn_minion_enemy};
use strum::{EnumCount, FromRepr};
use vleue_navigator::{prelude::{ManagedNavMesh, NavMeshStatus}, NavMesh, Path};
use weighted_rand::{builder::{NewBuilder, WalkerTableBuilder}, table::WalkerTable};

use crate::{arena::Ground, assets::EnemyAssets, enemy::{self, minion::MinionPlugin}, util::Health};

pub mod minion;

const MAX_ENEMIES: u32 = 1000;

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
        
            .add_systems(Update, (enemy_goto, enemy_idle_and_spawning).in_set(DefaultEnemyBehavior))
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
    AttackBeacon(Option<Path>, usize),
    AttackPlayer,
}

impl EnemyBehavior {
    pub fn attack_beacon() -> Self {
        EnemyBehavior::AttackBeacon(None, 0)
    }
    
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
    if enemy_count.0 > MAX_ENEMIES { return }
    
    for enemy in trigger.collect().into_iter() {
        commands.trigger(SpawnEnemy(trigger.0, enemy));
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
    mut enemy : Query<(&Transform, &mut TnuaController, &mut EnemyBehavior, &Enemy)>,
    navmesh : Single<(&ManagedNavMesh, &NavMeshStatus)>,
    navmeshes : Res<Assets<NavMesh>>
) {
    if *navmesh.1 != NavMeshStatus::Built { return };
    let Some(navmesh) = navmeshes.get(navmesh.0.id()) else { return };
    
    for (transform, mut controller, mut behavior, enemy) in enemy.iter_mut() {
        let EnemyBehavior::Goto(destination, current_path, index) = behavior.as_mut() else { continue; };
        let current_location = Vec2::new(transform.translation.x, transform.translation.z);
        if let None = current_path {
            let path = navmesh.path(current_location, *destination);
            *current_path = path;
        }
        
        let current_path = current_path.as_ref().unwrap();
        let current_node = current_path.path.get(*index).unwrap_or(destination);
        
        let direction_to_next = (current_node - current_location).normalize_or_zero();
        let move_2d = direction_to_next * enemy.speed;
        controller.basis(TnuaBuiltinWalk {
            desired_velocity: (move_2d.x, 0.0, move_2d.y).into(),
            float_height: enemy.height_from_ground,
            desired_forward: Some(Dir3::from_xyz_unchecked(direction_to_next.x, 0.0, direction_to_next.y)),
            ..default()
        });
        
        if current_location.distance(*current_node) < 0.3 && *index < current_path.path.len() {
            *index += 1;
        }
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

