use bevy::prelude::*;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt}};

use crate::GameState;

//==============================================================================================
//        Asset Plugin
//==============================================================================================

pub struct AssetLoadingPlugin;

impl Plugin for AssetLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::InGame)
                .load_collection::<WizardAssets>()
                .load_collection::<SpellAssets>()
                .load_collection::<EnemyAssets>()
                .load_collection::<BeaconAssets>()
        );
    }
}

//==============================================================================================
//        Spellbook Assets
//==============================================================================================

#[derive(AssetCollection, Resource)]
pub struct WizardAssets {
    #[asset(path = "models/wizard/spellbook_open.gltf#Scene0")]
    pub open: Handle<Scene>,
    #[asset(path = "models/wizard/spellbook_closed.gltf#Scene0")]
    pub closed: Handle<Scene>,
    #[asset(path = "models/wizard/Mage.glb#Scene0")]
    pub wizard: Handle<Scene>,
}

pub struct WizardAnimationGraph {
    pub graph: Handle<AnimationGraph>,
}

//==============================================================================================
//        Spell Assets
//==============================================================================================

#[derive(AssetCollection, Resource)]
pub struct SpellAssets {
    #[asset(path = "models/spell/dagger.gltf#Scene0")]
    pub dagger : Handle<Scene>
}

//==============================================================================================
//        Enemy Assets
//==============================================================================================

#[derive(AssetCollection, Resource)]
pub struct EnemyAssets {
    #[asset(path = "models/enemy/Skeleton_Minion.glb#Scene0")]
    pub skeleton_minion: Handle<Scene>,
    #[asset(path = "models/enemy/Skeleton_Mage.glb#Scene0")]
    pub skeleton_mage: Handle<Scene>,
}

//==============================================================================================
//        Beacon Assets
//==============================================================================================

#[derive(AssetCollection, Resource)]
pub struct BeaconAssets {
    #[asset(path = "models/beacon/beacon.glb#Scene0")]
    pub beacon: Handle<Scene>,
    #[asset(path = "models/beacon/beacon.glb#Animation0")]
    pub animation: Handle<AnimationClip>,
}
