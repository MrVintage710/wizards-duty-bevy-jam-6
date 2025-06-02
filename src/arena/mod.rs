use std::{f32::consts::FRAC_PI_2, ops::Deref};

use bevy::{app::DynEq, color::palettes, prelude::*, window::WindowResized};
use avian3d::prelude::*;
use vleue_navigator::{prelude::{ManagedNavMesh, NavMeshSettings, NavMeshStatus, NavMeshUpdateMode, NavmeshUpdaterPlugin}, NavMesh, NavMeshDebug, Triangulation};
use crate::GameState;

pub mod beacon;

const ARENA_SIZE: f32 = 500.0;

//==============================================================================================
//        ArenaPlugin
//==============================================================================================

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), build_arena);
        
        app.add_systems(Update, nav_mesh_done);
        if cfg!(debug_assertions) {
        }
    }
}

///This is a component that will mark an entity as something that should be navigated around
#[derive(Component)]
pub struct Obstacle;

#[derive(Component)]
pub struct ArenaProp;

#[derive(Component)]
pub struct Ground;

pub fn build_arena(
    mut commands : Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    previous_setup: Query<Entity, With<ArenaProp>>
) {
    
    for entity in previous_setup.iter() {
        commands.entity(entity).despawn();
    }
    
    let half_size = ARENA_SIZE / 2.0;
    
    // Spawn the ground
    commands.spawn((
        Name::new("Ground"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(ARENA_SIZE, ARENA_SIZE))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        ArenaProp,
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
        Ground,
    ));
    
    commands.spawn((
        Name::new("Nav Mesh"),
        NavMeshSettings{
            // Define the outer borders of the navmesh.
            fixed: Triangulation::from_outer_edges(&[
                vec2(-half_size, -half_size),
                vec2(half_size, -half_size),
                vec2(half_size, half_size),
                vec2(-half_size, half_size),
            ]),
            build_timeout: Some(1.0),
            simplify: 0.005,
            merge_steps: 0,
            ..default()
        },
        NavMeshDebug(palettes::tailwind::RED_800.into()),
        NavMeshUpdateMode::Direct,
        Transform::from_xyz(0.0, 0.1, 0.0).with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
    ));
    
    // Test Cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_xyz(1.5, 0.5, 1.5),
        Collider::cuboid(1.0, 1.0, 1.0),
        RigidBody::Static,
        Obstacle,
        ArenaProp
    ));
    
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_xyz(1.5, 3.0, 1.5),
        Collider::cuboid(1.0, 1.0, 1.0),
        RigidBody::Static,
        ArenaProp,
    ));
}

//==============================================================================================
//        For Debuging the navmesh
//==============================================================================================

fn nav_mesh_done(
    navmesh: Single<Option<&NavMeshStatus>>,
) {
    println!("Navmesh Status: {:?}", *navmesh);
}
