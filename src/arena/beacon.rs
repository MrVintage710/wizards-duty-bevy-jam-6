
use avian3d::prelude::{Collider, RigidBody};
use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{arena::{ArenaProp, NavmeshQuery, Obstacle}, assets::BeaconAssets, util::{GameInit, Health, SceneRootWithAnimation}, GameState};

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
        Health::new(1000.0),
        SceneRootWithAnimation::new(assets.beacon.clone())
            .with_animation_graph(graphs.add(graph))
            .with_animation(id)
            .repeat()
    ));
}

//==============================================================================================
//        Beacon Components
//==============================================================================================

#[derive(Component)]
pub struct Beacon;

//==============================================================================================
//        Beceaon Util
//==============================================================================================

const CLOSEST_POINT_RADIUS_DEFAULT: f32 = 2.0;

#[derive(SystemParam)]
pub struct BeaconQuery<'w> {
    pub beacon: Single<'w, (&'static Transform, &'static mut Health), With<Beacon>>,
}

impl<'w> BeaconQuery<'w> {
    
    pub fn closest_point(&self, other : &Transform, range : f32) -> Vec2 {
        let other = other.translation.xz();
        let beacon = self.beacon.0.translation.xz();
        
        let inbetween = -(beacon - other).normalize();
        let closest = inbetween * range;
        
        closest
    }

    pub fn towards_beacon(&self, other : &Transform) -> Dir3 {
        let direction = (other.translation.xz() - self.beacon.0.translation.xz()).normalize_or_zero();
        Dir3::from_xyz_unchecked(direction.x, 0.0, direction.y)
    }

    pub fn take_damage(&mut self, damage : f32) {
        self.beacon.1.take_damage(damage);
    }
    
    pub fn within_range(&self, other : &Transform, range : f32) -> bool {
        let other = other.translation.xz();
        let beacon = self.beacon.0.translation.xz();
        
        let distance = (beacon - other).length();
        
        distance <= range
    }
}
