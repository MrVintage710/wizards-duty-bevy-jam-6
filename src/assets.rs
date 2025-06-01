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
