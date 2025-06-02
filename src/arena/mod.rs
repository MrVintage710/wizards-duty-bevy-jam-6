use bevy::prelude::*;
use avian3d::prelude::*;
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
    }
}

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
    
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(ARENA_SIZE, ARENA_SIZE))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        ArenaProp,
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
        Ground
    ));
    
    // Test Cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_xyz(1.5, 0.5, 1.5),
        Collider::cuboid(1.0, 1.0, 1.0),
        RigidBody::Static,
        ArenaProp
    ));
    
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_xyz(1.5, 3.0, 1.5),
        Collider::cuboid(1.0, 1.0, 1.0),
        RigidBody::Static,
        ArenaProp
    ));
}
