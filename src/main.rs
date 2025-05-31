use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use render::{pixelate::PixelationEffect, RenderPhase};

pub mod render;

fn main() -> AppExit {
    App::new()
        .register_type::<PixelationEffect>()
        
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
        .add_plugins(WorldInspectorPlugin::new())
        // .add_plugins(PixelCamPlugin)
        .add_plugins(PixelationEffect::plugin())

        .add_systems(Startup, setup)

    .run()
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    // mut pixel_cam_transform : Single<&mut Transform, With<PixelCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // camera
        commands.spawn((
            Name::new("Main Camera"),
            Camera3d::default(),
            Projection::from(OrthographicProjection {
                // 6 world units per pixel of window height.
                scaling_mode: ScalingMode::FixedVertical { viewport_height: 6.0 },
                ..OrthographicProjection::default_3d()
            }),
            PixelationEffect::default(),
            Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ));
    
        // plane
        commands.spawn((
            Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        ));
        // cubes
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::default())),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
            Transform::from_xyz(1.5, 0.5, 1.5),
        ));
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::default())),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
            Transform::from_xyz(1.5, 0.5, -1.5),
        ));
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::default())),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
            Transform::from_xyz(-1.5, 0.5, 1.5),
        ));
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::default())),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
            Transform::from_xyz(-1.5, 0.5, -1.5),
        ));
        
        // light
        commands.spawn((PointLight::default(), Transform::from_xyz(3.0, 8.0, 5.0)));
}
