
use bevy::{pbr::Atmosphere, prelude::*, render::camera::ScalingMode};
use bevy_inspector_egui::InspectorOptions;

use crate::{render::pixelate::PixelationEffect, util::IsometricPosition};

//==============================================================================================
//        CameraPlugin
//==============================================================================================


pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<CameraFocus>()
            .add_systems(PreStartup, spawn_camera)
            .add_systems(PreUpdate, camera_follow)
        ;
    }
}

//==============================================================================================
//        Components
//==============================================================================================

#[derive(Component)]
pub struct MainCamera;

#[derive(Component, Reflect, InspectorOptions)]
#[reflect(Component, Default)]
#[require(Transform)]
pub struct CameraFocus {
    pub speed: f32,
    pub rotation: f32,
    #[inspector(min = 0.0, max = 1.0)]
    pub zoom: f32,
}

impl Default for CameraFocus {
    fn default() -> Self {
        Self {
            speed: 6.0,
            rotation: 45.0,
            zoom: 0.0,
        }
    }
}

#[derive(Component)]
#[require(Transform)]
pub struct CameraTarget;

//==============================================================================================
//        Systems
//==============================================================================================

const CAMERA_DISTANCE: f32 = 10.0;

pub fn camera_follow(
    focus : Single<(&mut Transform, &CameraFocus), Without<CameraTarget>>,
    target: Single<&Transform, With<CameraTarget>>,
    mut cam : Single<&mut Projection, With<MainCamera>>,
    time : Res<Time>
) {
    let (mut camera_focus_transform, cam_focus) = focus.into_inner();
    camera_focus_transform.translation = camera_focus_transform.translation.move_towards(target.translation, cam_focus.speed * time.delta_secs());
    camera_focus_transform.rotation = Quat::from_rotation_y(cam_focus.rotation.to_radians());
    
    if let Projection::Orthographic(ortho) = cam.as_mut() {
        if let ScalingMode::FixedVertical { viewport_height } = &mut ortho.scaling_mode {
            *viewport_height = 6.0 + (6.0 * cam_focus.zoom)
        }
    }
}

pub fn spawn_camera(
    mut commands: Commands,
) {
    let camera_focus = CameraFocus::default();
    
    let mut camera_transform = Transform::default();
    let rotation = Quat::from_euler(EulerRot::YXZ, 0.0, -35.264_f32.to_radians(), 0.0);
    camera_transform.translation = -(rotation.mul_vec3(Vec3::NEG_Z) * CAMERA_DISTANCE);
    camera_transform.rotation = rotation;
    
    commands.spawn((
        Name::new("Camera Focus"),
        camera_focus,
        children![
            (
                Name::new("Main Camera"),
                Camera3d::default(),
                Projection::from(OrthographicProjection {
                    // 6 world units per pixel of window height.
                    scaling_mode: bevy::render::camera::ScalingMode::FixedVertical { viewport_height: 6.0 },
                    ..OrthographicProjection::default_3d()
                }),
                PixelationEffect::default(),
                camera_transform,
                MainCamera,
            )
        ]
    ));
    
    // commands.spawn((
    //     Name::new("Camera Target"), 
    //     CameraTarget)
    // );
}
