use arena::ArenaPlugin;
use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_enhanced_input::EnhancedInputPlugin;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use camera::CameraPlugin;
use character::PlayerCharacterPlugin;
use render::{pixelate::PixelationEffect, RenderPhase};
use util::IsometricPositionPlugin;

pub mod render;
pub mod arena;
pub mod character;
pub mod camera;
pub mod util;

fn main() -> AppExit {
    let mut app = App::new();
    app
        .register_type::<PixelationEffect>()
        
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(EnhancedInputPlugin)
        
        // Plugin that gives the pixelation effect to the camera
        .add_plugins(PixelationEffect::plugin())
        
        // Plugin for the Play area
        .add_plugins(ArenaPlugin)
        
        // This spawns and manages the camera
        .add_plugins(CameraPlugin)
        
        // This is used to calcualte all of the iso positions
        .add_plugins(IsometricPositionPlugin)
        
        .add_plugins(PlayerCharacterPlugin)
        
        .add_systems(Startup, setup)
    ;
    
    // All plugins that are only used in non release builds
    if cfg!(debug_assertions) {
        app
            .add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
            .add_plugins(WorldInspectorPlugin::new())
        ;
    }
    
    app.run()
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    // mut pixel_cam_transform : Single<&mut Transform, With<PixelCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // camera
        // commands.spawn((
        //     Name::new("Main Camera"),
        //     Camera3d::default(),
        //     Projection::from(OrthographicProjection {
        //         // 6 world units per pixel of window height.
        //         scaling_mode: ScalingMode::FixedVertical { viewport_height: 6.0 },
        //         ..OrthographicProjection::default_3d()
        //     }),
        //     PixelationEffect::default(),
        //     Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        // ));
        
        let directional_light = DirectionalLight {
            color: Color::Srgba(Srgba::rgba_u8(138, 135, 245, 255)),
            shadows_enabled: true,
            ..Default::default()
        };
        
        // light
        commands.spawn((
            directional_light,
            Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, 5.5, 1.0, 0.0))
        ));
        // commands.spawn((PointLight::default(), Transform::from_xyz(3.0, 8.0, 5.0)));
}
