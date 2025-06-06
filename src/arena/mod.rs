use std::f32::consts::FRAC_PI_2;
use bevy::{color::palettes, ecs::system::SystemParam, prelude::*};
use avian3d::prelude::*;
use vleue_navigator::{prelude::{ManagedNavMesh, NavMeshSettings, NavMeshStatus, NavMeshUpdateMode}, NavMesh, NavMeshDebug, Triangulation};
use crate::{arena::beacon::{spawn_beacon, BeaconPlugin}, GameState};
use vleue_navigator::{prelude::*, Path};

pub mod beacon;

pub const ARENA_SIZE: f32 = 50.0;

//==============================================================================================
//        ArenaPlugin
//==============================================================================================

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(BeaconPlugin)
            .add_systems(OnEnter(GameState::InGame), build_arena);
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
            merge_steps: 1,
            agent_radius : 0.6,
            ..default()
        },
        NavMeshDebug(palettes::tailwind::RED_800.into()),
        NavMeshUpdateMode::Direct,
        Transform::from_xyz(0.0, 0.1, 0.0).with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
    ));
}

//==============================================================================================
//        Navemsh Query Param
//==============================================================================================

#[derive(SystemParam)]
pub struct NavmeshQuery<'w> {
    navmesh : Single<'w, (&'static ManagedNavMesh, &'static NavMeshStatus)>,
    navmeshes : Res<'w, Assets<NavMesh>>
}

impl<'w> NavmeshQuery<'w> {
    pub fn new(navmesh: Single<'w, (&'static ManagedNavMesh, &'static NavMeshStatus)>, navmeshes: Res<'w, Assets<NavMesh>>) -> Self {
        Self { navmesh, navmeshes }
    }
    
    pub fn path_from_tranform(&self, from : &Transform, to : Vec2) -> Option<Path> {
        if *self.navmesh.1 != NavMeshStatus::Built { return None};
        let Some(navmesh) = self.navmeshes.get(self.navmesh.0.id()) else { return None};
        navmesh.path(from.translation.xz(), to)
    }
}
