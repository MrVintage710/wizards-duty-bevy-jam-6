use bevy::prelude::*;
use minion::{minion_behavior, spawn_minion_enemy};
use strum::{EnumCount, FromRepr};
use vleue_navigator::Path;
use weighted_rand::{builder::{NewBuilder, WalkerTableBuilder}, table::WalkerTable};

use crate::assets::EnemyAssets;

pub mod minion;

const MAX_ENEMIES: u32 = 1000;

//==============================================================================================
//        Enemy Plguin
//==============================================================================================

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<EnemyCount>()
            
            .add_observer(spawn_enemy)
        
            .add_systems(Update, (minion_behavior, debug_paths).chain())
        ;
    }
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

#[derive(Component, Default, Debug)]
pub enum EnemyBehavior {
    #[default]
    None,
    Guard,
    AttackBeacon(Option<Path>, usize),
    AttackPlayer,
}

impl EnemyBehavior {
    pub fn attack_beacon() -> Self {
        EnemyBehavior::AttackBeacon(None, 0)
    }
}

//==============================================================================================
//         Spawn Enemy Event
//==============================================================================================

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

pub fn spawn_enemy(
    trigger : Trigger<SpawnEnemiesEvent>,
    mut commands : Commands,
    mut enemy_count: ResMut<EnemyCount>,
    enemy_assets : Res<EnemyAssets>,
) {
    if enemy_count.0 > MAX_ENEMIES { return }
    
    for enemy in trigger.collect().into_iter() {
        match enemy {
            EnemyType::Minion => spawn_minion_enemy(&mut commands, trigger.0, enemy_assets.as_ref()),
            EnemyType::Mage => todo!(),
        }
        enemy_count.0 += 1;
    }
}

pub fn debug_paths (
    enemy_behaviors : Query<(&EnemyBehavior, &Transform)>,
    mut gizmos : Gizmos
) {
    for (enemy_behavior, transform) in enemy_behaviors.iter() {
        if let EnemyBehavior::AttackBeacon(Some(path), index) = enemy_behavior {
            let mut points : Vec<Vec3> = vec![(transform.translation.x, 0.1, transform.translation.z).into()];
            path.path.iter().for_each(|point| points.push(Vec3::new(point.x, 0.1, point.y)));
            if let Some(point) = points.get(*index + 1) {
                gizmos.sphere(Isometry3d::from_translation(*point), 0.3, bevy::color::palettes::tailwind::EMERALD_600);
            }
            gizmos.linestrip(points, bevy::color::palettes::tailwind::EMERALD_600);
        }
    }
}

