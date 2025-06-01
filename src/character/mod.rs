use aim::AimPlugin;
use bevy::{input::mouse::MouseWheel, prelude::*};
use bevy_enhanced_input::prelude::*;

use crate::{assets::WizardAssets, camera::{CameraFocus, CameraTarget}, GameState};

pub mod aim;

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
            .add_observer(move_character)
            .add_observer(zoom_cam)
            
            .add_systems(OnEnter(GameState::InGame), setup_player)
        ;
    }
}

//==============================================================================================
//        PlayerCharacter
//==============================================================================================

#[derive(Component)]
#[require(Transform)]
pub struct PlayerCharacter {
    speed: f32,
}

impl Default for PlayerCharacter {
    fn default() -> Self {
        PlayerCharacter {
            speed: 4.0,
        }
    }
}

//==============================================================================================
//        Bind Key Actions
//==============================================================================================

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
pub struct Move;

#[derive(Debug, InputAction)]
#[input_action(output = f32)]
pub struct Zoom;

#[derive(InputContext)]
pub struct OnFoot;

pub fn bind_actions(
    trigger: Trigger<Binding<OnFoot>>,
    mut actions: Query<&mut Actions<OnFoot>>
) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<Move>().to((Cardinal::wasd_keys(), Axial::left_stick()));
    actions.bind::<Zoom>().to((Input::mouse_wheel(), GamepadAxis::LeftStickY));
}

//==============================================================================================
//        Character Controls
//==============================================================================================

fn move_character(
    trigger : Trigger<Fired<Move>>,
    mut query : Query<(&mut Transform, &PlayerCharacter)>,
    cam_focus : Single<&CameraFocus>,
    time : Res<Time>
) {
    let (mut player_transform, player_character) = query.get_mut(trigger.target()).unwrap();
    let move_value = Vec3::new(trigger.value.x, 0.0, -trigger.value.y).normalize();
    let rotation = Quat::from_euler(EulerRot::XYZ, 0.0, cam_focus.rotation.to_radians(), 0.0);
    let rotated_move = rotation.mul_vec3(move_value);
    player_transform.translation += rotated_move * player_character.speed * time.delta().as_secs_f32();
}

fn zoom_cam(
    trigger : Trigger<Fired<Zoom>>,
    mut camera_focus : Single<&mut CameraFocus>,
    time : Res<Time>
) {
    let value = trigger.value;
    camera_focus.zoom = (camera_focus.zoom + value * 0.2 * time.delta_secs()).clamp(0.0, 1.0);
}

//==============================================================================================
//        Systems
//==============================================================================================

pub fn setup_player(
    mut commands : Commands,
    wizards_assets : Res<WizardAssets>
) {
    commands.spawn((
        Name::new("Player"),
        PlayerCharacter::default(),
        Transform::from_xyz(0.0, 0.0, 0.0),
        CameraTarget,
        Actions::<OnFoot>::default(),
        SceneRoot(wizards_assets.wizard.clone())
    ));
}



