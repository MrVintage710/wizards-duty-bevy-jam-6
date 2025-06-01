use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::camera::{CameraFocus, CameraTarget};

//==============================================================================================
//        PlayerCharacterPlugin
//==============================================================================================

pub struct PlayerCharacterPlugin;

impl Plugin for PlayerCharacterPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_input_context::<OnFoot>()
            
            .add_observer(bind_actions)
            .add_observer(move_character)
            
            .add_systems(Startup, setup_player)
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
//        Systems
//==============================================================================================

pub fn setup_player(
    mut commands : Commands,
    mut meshes : ResMut<Assets<Mesh>>,
    mut materials : ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Name::new("Player"),
        PlayerCharacter::default(),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Mesh3d(meshes.add(Cylinder::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        CameraTarget,
        Actions::<OnFoot>::default()
    ));
}

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
pub struct Move;

#[derive(InputContext)]
pub struct OnFoot;

pub fn bind_actions(
    trigger: Trigger<Binding<OnFoot>>,
    mut actions: Query<&mut Actions<OnFoot>>
) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<Move>().to((Cardinal::wasd_keys(), Axial::left_stick()));
}

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

