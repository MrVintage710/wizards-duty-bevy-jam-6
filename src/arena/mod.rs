use bevy::prelude::*;

const ARENA_SIZE: f32 = 1000.0;

//==============================================================================================
//        ArenaPlugin
//==============================================================================================

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, build_arena);
    }
}

#[derive(Component)]
pub struct ArenaProp;

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
    ));
    
    // Test Cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_xyz(1.5, 0.5, 1.5),
        ArenaProp
    ));
    
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_xyz(1.5, 3.0, 1.5),
        ArenaProp
    ));
}
