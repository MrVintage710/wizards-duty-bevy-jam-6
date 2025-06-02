use bevy::prelude::*;
use minion::spawn_minion_enemy;
use strum::{EnumCount, FromRepr};
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
        todo!()
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
            number_of_enemies : 3,
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
    enemy_assets : Res<EnemyAssets>,
    enemy_count: Res<EnemyCount>
) {
    if enemy_count.0 > MAX_ENEMIES { return }
    
    for enemy in trigger.collect().into_iter() {
        match enemy {
            EnemyType::Minion => spawn_minion_enemy(&mut commands, trigger.0, enemy_assets.as_ref()),
            EnemyType::Mage => todo!(),
        }
    }
}


