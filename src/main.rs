use arena::{ArenaPlugin, Obstacle};
use assets::{AssetLoadingPlugin, WizardAssets};
use bevy::prelude::*;
use bevy_enhanced_input::EnhancedInputPlugin;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::*;
use camera::CameraPlugin;
use character::PlayerCharacterPlugin;
use enemy::{EnemyPlugin, EnemyType, SpawnEnemiesEventBuilder};
use render::{pixelate::PixelationEffect, RenderPhase};
use spells::SpellPlugin;
use avian3d::prelude::*;
use vleue_navigator::prelude::*;

pub mod render;
pub mod arena;
pub mod character;
pub mod camera;
pub mod util;
pub mod assets;
pub mod spells;
pub mod enemy;

//==============================================================================================
//        GameState
//==============================================================================================

#[derive(States, Default, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum GameState {
    #[default]
    Loading,
    InGame,
}

//==============================================================================================
//        Main Function
//==============================================================================================

fn main() -> AppExit {
    let mut app = App::new();
    app
        .register_type::<PixelationEffect>()
        
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(MeshPickingPlugin)
        .add_plugins(EnhancedInputPlugin)
        .add_plugins(PhysicsPlugins::default())
        .add_plugins((TnuaControllerPlugin::new(FixedUpdate), TnuaAvian3dPlugin::new(FixedUpdate)))
        
        .add_plugins(VleueNavigatorPlugin)
        .add_plugins(NavmeshUpdaterPlugin::<Collider, Obstacle>::default())
        
        //This is where all of the assets are loaded
        .add_plugins(AssetLoadingPlugin)
        
        // Plugin that gives the pixelation effect to the camera
        .add_plugins(PixelationEffect::plugin())
        
        // Plugin for the Play area
        .add_plugins(ArenaPlugin)
        
        // This spawns and manages the camera
        .add_plugins(CameraPlugin)
        
        //This has everything to do with the player character, including movement.
        .add_plugins(PlayerCharacterPlugin)
        
        //This will have everything needed for the spells to work
        .add_plugins(SpellPlugin)
        
        //This is where all of the enemy logic is.
        .add_plugins(EnemyPlugin)
        
        .add_systems(OnEnter(GameState::InGame), setup)
        
        .init_state::<GameState>()
    ;
    
    // All plugins that are only used in non release builds
    if cfg!(debug_assertions) {
        app
            .add_plugins(EguiPlugin { enable_multipass_for_primary_context: true })
            .add_plugins(WorldInspectorPlugin::new())
            // .add_plugins(PhysicsDebugPlugin::default())
            .insert_gizmo_config(PhysicsGizmos::default(), GizmoConfig::default())
        ;
    }
    
    app.run()
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    wizards_assets: Res<WizardAssets>,
) {
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
    
    commands.spawn((
        Name::new("Spellbook"),
        Transform::from_xyz(0.0, 2.5, 0.0),
        SceneRoot(wizards_assets.closed.clone())
    ));
    
    commands.trigger(SpawnEnemiesEventBuilder::new((25.0, 0.0, 25.0).into()).with_weight(EnemyType::Minion, 1).build());
}


