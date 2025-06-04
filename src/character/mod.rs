use std::f64::consts::PI;

use aim::{AimPlugin, ShootTarget};
use avian3d::{dynamics::rigid_body, prelude::{Collider, Friction, LinearVelocity, LockedAxes, RigidBody}};
use bevy::{input::mouse::MouseWheel, math::VectorSpace, prelude::*};
use bevy_enhanced_input::prelude::*;
use bevy_tnua::{controller, prelude::{TnuaBuiltinWalk, TnuaController}};
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;

use crate::{assets::WizardAssets, camera::{CameraFocus, CameraTarget}, spells::{CastSpell, Spellbook}, GameState};

pub mod aim;

pub const PLAYER_HEALTH: f32 = 100.0;
pub const PLAYER_SPEED: f32 = 4.0;

//==============================================================================================
//        PlayerCharacterPlugin
//==============================================================================================

pub struct PlayerCharacterPlugin;

impl Plugin for PlayerCharacterPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(AimPlugin)
            
            .add_input_context::<OnFoot>()
            
            .add_observer(bind_actions)
            .add_observer(zoom_cam)
            .add_observer(cast_spell)
            
            .add_systems(OnEnter(GameState::InGame), setup_player)
            .add_systems(Update, move_character)
        ;
    }
}

//==============================================================================================
//        PlayerCharacter
//==============================================================================================

#[derive(Component)]
#[require(Transform, Name::new("ShootOrigin"))]
pub struct ShootOrigin;

#[derive(Component)]
#[require(Transform)]
pub struct PlayerCharacter {
    health: f32,
    speed: f32,
}

#[derive(Component)]
pub struct PlayerModelRoot;

impl Default for PlayerCharacter {
    fn default() -> Self {
        PlayerCharacter {
            health: PLAYER_HEALTH,
            speed: PLAYER_SPEED,
        }
    }
}

//==============================================================================================
//        Bind Key Actions
//==============================================================================================

const AIM_HEIGHT: f32 = 1.0;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
pub struct Move;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
pub struct Zoom;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct EvokeSpell;

#[derive(InputContext)]
pub struct OnFoot;

pub fn bind_actions(
    trigger: Trigger<Binding<OnFoot>>,
    mut actions: Query<&mut Actions<OnFoot>>
) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<Move>().to((Cardinal::wasd_keys(), Axial::left_stick()));
    actions.bind::<Zoom>().to((Input::mouse_wheel(), GamepadAxis::LeftStickY));
    actions.bind::<EvokeSpell>().to((MouseButton::Left, GamepadButton::RightTrigger));
}

//==============================================================================================
//        Character Controls
//==============================================================================================

fn move_character(
    query : Single<(&mut TnuaController, &Actions<OnFoot>, &PlayerCharacter)>,
    cam_focus : Single<&CameraFocus>,
    mut forward_vec : Local<Vec3>
) {
    let (mut controller, actions, player_character) = query.into_inner();
    
    let mut move_vector = Vec3::ZERO;
    
    if actions.state::<Move>().unwrap() == ActionState::Fired {
        let in_move = actions.value::<Move>().unwrap().as_axis2d();
        let move_value = Vec3::new(in_move.x, 0.0, -in_move.y).normalize_or_zero();
        let rotation = Quat::from_euler(EulerRot::XYZ, 0.0, cam_focus.rotation.to_radians(), 0.0);
        let rotated_move = rotation.mul_vec3(move_value);
        *forward_vec = rotated_move;
        move_vector = rotated_move * player_character.speed;
    }
    
    if *forward_vec == Vec3::default() {
        *forward_vec = Vec3::new(0.0, 0.0, -1.0);
    }
    
    controller.basis(TnuaBuiltinWalk {
        desired_velocity: move_vector,
        float_height: 1.0,
        max_slope: 45.0_f32.to_radians(),
        desired_forward : Some(Dir3::from_xyz_unchecked(forward_vec.x, forward_vec.y, forward_vec.z)),
        ..default()
    });
}

fn zoom_cam(
    trigger : Trigger<Fired<Zoom>>,
    mut camera_focus : Single<&mut CameraFocus>,
    time : Res<Time>
) {
    let value = trigger.value;
    camera_focus.zoom = (camera_focus.zoom + value.y * 0.2 * time.delta_secs()).clamp(0.0, 1.0);
}

fn cast_spell (
    _trigger : Trigger<Fired<EvokeSpell>>,
    mut commands : Commands,
    shoot_origin : Single<&Transform, With<ShootOrigin>>,
    shoot_target : Single<&Transform, With<ShootTarget>>,
    spellbook : Res<Spellbook>,
    mut gizmos : Gizmos
) {
    let target_2d = Vec2::new(shoot_target.translation.x, shoot_target.translation.z);
    let origin_2d = Vec2::new(shoot_origin.translation.x, shoot_origin.translation.z);
    let direction = (target_2d - origin_2d).normalize_or_zero();    gizmos.ray(shoot_origin.translation, Vec3::new(direction.x, 0.0, direction.y), Color::srgb(1.0, 0.0, 0.0));
    if spellbook.cooldown.finished() {
        commands.trigger(CastSpell {
            position: shoot_origin.translation,
            direction,
            spell_index: 0,
        });   
    }
}

//==============================================================================================
//        Systems
//==============================================================================================

pub fn setup_player(
    mut commands : Commands,
    wizards_assets : Res<WizardAssets>
) {
    commands.spawn((
        ShootOrigin, 
        Transform::from_translation(Vec3::new(-2.5, AIM_HEIGHT, 0.0)), 
        CameraTarget,
        RigidBody::Dynamic,
        Collider::capsule(0.5, 0.5),
        Actions::<OnFoot>::default(),
        PlayerCharacter::default(),
        Name::new("Player"),
        
        TnuaController::default(),
        // A sensor shape is not strictly necessary, but without it we'll get weird results.
        TnuaAvian3dSensorShape(Collider::cylinder(0.49, 0.0)),
        children![
            (
                Transform::from_xyz(0.0, -AIM_HEIGHT, 0.0).with_rotation(Quat::from_rotation_y(PI as f32)),
                SceneRoot(wizards_assets.wizard.clone()),
                PlayerModelRoot
            )
        ]
    ));
}



