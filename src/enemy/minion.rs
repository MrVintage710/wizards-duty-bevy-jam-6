
use bevy::prelude::*;
use crate::assets::EnemyAssets;

//==============================================================================================
//        Spawn a minion Enemy
//==============================================================================================


pub fn spawn_minion_enemy(commands : &mut Commands, position : Vec3, enemy_assets : &EnemyAssets) {
    
    commands.spawn((
        Transform::from_translation(position),
        Minion,
        
    ));
    // Implementation details here
}

//==============================================================================================
//        Minion Logic
//==============================================================================================

#[derive(Component)]
pub struct Minion;