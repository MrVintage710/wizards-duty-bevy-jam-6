
use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

use crate::{arena::{ArenaProp, Obstacle}, assets::BeaconAssets, util::{init_scene_root_with_animation, DefaultAnimationGraphMap, GameInit, SceneRootWithAnimation}, GameState};

//==============================================================================================
//        Beacon Plugin
//==============================================================================================

pub struct BeaconPlugin;

impl Plugin for BeaconPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::InGame), spawn_beacon.in_set(GameInit))
        ;
    }
}

//==============================================================================================
//        Spawn Beacon System
//==============================================================================================


pub fn spawn_beacon(
    mut commands : Commands,
    mut graphs : ResMut<Assets<AnimationGraph>>,
    assets: Res<BeaconAssets>,
) {
    
    let (graph, id) = AnimationGraph::from_clip(assets.animation.clone());
    
    commands.spawn((
        Name::new("Beacon"),
        Beacon,
        ArenaProp,
        Obstacle,
        Collider::cuboid(1.0, 4.0, 1.0),
        RigidBody::Static,
        Transform::from_rotation(Quat::from_rotation_y(-45.0_f32.to_radians())),
        SceneRootWithAnimation::new(assets.beacon.clone(), graphs.add(graph))
            .with_animation(id)
            .repeat()
    ));
}

//==============================================================================================
//        Beacon Components
//==============================================================================================

#[derive(Component)]
pub struct Beacon;

// fn init_beacon_animations(
//     mut commands: Commands,
//     mut query: Query<(Entity, &mut AnimationPlayer), With<Beacon>>,
//     mut graphs : ResMut<Assets<AnimationGraph>>,
//     assets : Res<BeaconAssets>,
//     mut done: Local<bool>,
// ) {
//     if *done {
//         return;
//     }

//     for (entity, mut player) in query.iter_mut() {
//         let (graph, animation_index) = AnimationGraph::from_clip(assets.animation.clone());
        
//         commands.entity(entity).insert(
//             AnimationGraphHandle(graphs.add(graph))
//         );
    
//         player.play(animation_index).repeat();
    

//         *done = true;
//     }
// }