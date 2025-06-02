
use avian3d::prelude::{CollisionLayers, PhysicsLayer};
use bevy::prelude::*;

//==============================================================================================
//        Collision Layers for the Game
//==============================================================================================

#[derive(PhysicsLayer, Default)]
pub enum GameCollisionLayer {
    #[default]
    Default,
    Player,
    Enemy,
    Projectile,
    Wall,
}
