use std::{collections::{HashMap, HashSet}, hash::Hash, u64};

use bevy::{animation::{graph::AnimationNodeIndex, AnimationTargetId}, prelude::*};
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt}};
use bevy_seedling::sample::Sample;

use crate::{assets, enemy::{minion::MinionStabbed, EnemySpawnAnimationComplete}, GameState};

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
                .finally_init_resource::<EnemyAnimationGraphs>()
        )
        .add_systems(OnExit(GameState::Loading), add_events_to_animations)
        ;
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
    #[asset(path = "sounds/oof1.ogg")]
    pub oof1: Handle<Sample>,
    #[asset(path = "sounds/oof2.ogg")]
    pub oof2: Handle<Sample>,
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
    #[asset(path = "models/enemy/Skeleton_Minion.glb#Animation41")]
    pub skeleton_minion_idle: Handle<AnimationClip>,
    #[asset(path = "models/enemy/Skeleton_Minion.glb#Animation55")]
    pub skeleton_minion_running: Handle<AnimationClip>,
    #[asset(path = "models/enemy/Skeleton_Minion.glb#Animation57")]
    pub skeleton_minion_running_fast: Handle<AnimationClip>,
    #[asset(path = "models/enemy/Skeleton_Minion.glb#Animation74")]
    pub skeleton_minion_spawn: Handle<AnimationClip>,
    #[asset(path = "models/enemy/Skeleton_Minion.glb#Animation39")]
    pub skeleton_minion_hit: Handle<AnimationClip>,
    #[asset(path = "models/enemy/Skeleton_Minion.glb#Animation3")]
    pub skeleton_minion_stab: Handle<AnimationClip>,
    
    #[asset(path = "models/enemy/Skeleton_Mage.glb#Scene0")]
    pub skeleton_mage: Handle<Scene>,
    
    #[asset(path = "models/enemy/Skeleton_Blade.gltf#Scene0")]
    pub skeleton_blade: Handle<Scene>,
}

#[derive(Resource)]
pub struct EnemyAnimationGraphs {
    pub minion_graph: Handle<AnimationGraph>,
    pub minion_idle: AnimationNodeIndex,
    pub minion_run_top: AnimationNodeIndex,
    pub minion_run_bottom: AnimationNodeIndex,
    pub minion_run_fast: AnimationNodeIndex,
    pub minion_spawn: AnimationNodeIndex,
    pub minion_stab: AnimationNodeIndex,
}

const MINION_UPPER: [&str; 4] = [
    "Rig/root/hips/spine",
    "chest/upperarm.l/lowerarm.l/wrist.l/hand.l/handslot.l",
    "chest/upperarm.r/lowerarm.r/wrist.r/hand.r/handslot.r",
    "chest/head",
];

const MINION_LOWER: [&str; 3] = [
    "Rig/root/hips/spine",
    "upperleg.l/lowerleg.l/foot.l/toes.l",
    "upperleg.r/lowerleg.r/foot.r/toes.r",
];

impl FromWorld for EnemyAnimationGraphs {
    fn from_world(world: &mut World) -> Self {
        let mut graph = AnimationGraph::new();
        let assets = world.resource::<EnemyAssets>();
    
        add_bones_mask(&mut graph, &MINION_UPPER, 0);
        add_bones_mask(&mut graph, &MINION_LOWER, 1);
    
        let minion_idle = graph.add_clip(assets.skeleton_minion_idle.clone(), 1.0, graph.root);
        let minion_run_bottom = graph.add_clip_with_mask(assets.skeleton_minion_running.clone(), 0b01, 1.0, graph.root);
        let minion_run_top = graph.add_clip_with_mask(assets.skeleton_minion_running.clone(), 0b10, 1.0, graph.root);
        let minion_run_fast = graph.add_clip(assets.skeleton_minion_running_fast.clone(), 1.0, graph.root);
        let minion_spawn = graph.add_clip(assets.skeleton_minion_spawn.clone(), 1.0, graph.root);
        let minion_stab = graph.add_clip_with_mask(assets.skeleton_minion_stab.clone(), 0b10, 1.0, graph.root);
    
        EnemyAnimationGraphs {
            minion_graph: world.resource_mut::<Assets<AnimationGraph>>().add(graph),
            minion_idle,
            minion_run_top,
            minion_run_bottom,
            minion_run_fast,
            minion_spawn,
            minion_stab,
        }
    }
}

fn add_bones_mask(graph : &mut AnimationGraph, bone_paths : &'static [&str], mask_index : u32) -> HashSet<AnimationTargetId> {
    let mut animation_targets = HashSet::new();
    let prefix = bone_paths[0];
    let prefix: Vec<_> = prefix.split('/').map(Name::new).collect();
    for path in bone_paths.iter().skip(1) {
        let suffix: Vec<_> = path.split('/').map(Name::new).collect();
        
        for chain_length in 0..=suffix.len() {
            let names = prefix.iter().chain(suffix[0..chain_length].iter()).collect::<Vec<_>>();
            println!("{names:?}");
            let animation_target_id = AnimationTargetId::from_names(names.into_iter());
            animation_targets.insert(animation_target_id);
            graph.add_target_to_mask_group(animation_target_id, mask_index);
        }
    }
    animation_targets
}

fn add_events_to_animations(
    mut animation_clips : ResMut<Assets<AnimationClip>>,
    enemy_assets : Res<EnemyAssets>,
) {
    let minion_attack_animation_clip = animation_clips.get_mut(&enemy_assets.skeleton_minion_stab).unwrap();
    minion_attack_animation_clip.add_event(minion_attack_animation_clip.duration() * 0.25, MinionStabbed);
    
    let minion_spawn_animation_clip = animation_clips.get_mut(&enemy_assets.skeleton_minion_spawn).unwrap();
    minion_spawn_animation_clip.add_event(minion_spawn_animation_clip.duration(), EnemySpawnAnimationComplete);
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
