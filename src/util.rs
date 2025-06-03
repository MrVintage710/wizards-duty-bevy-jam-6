
use std::collections::HashMap;

use avian3d::prelude::PhysicsLayer;
use bevy::{ecs::schedule::ScheduleLabel, prelude::*, scene::{InstanceId, SceneInstanceReady}, state::commands};

use crate::GameState;

//==============================================================================================
//        InitGame Schedule
//==============================================================================================

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameInit;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PostGameInit;

//==============================================================================================
//        Collision Layers for the Game
//==============================================================================================

#[derive(PhysicsLayer, Default)]
pub enum GameCollisionLayer {
    #[default]
    Default,
    Player,
    Enemy,
    Spell,
    Obstacle
}

//==============================================================================================
//        DefaultAnimationGraph
//==============================================================================================


pub struct DefaultSceneAnimationPlugin;

impl Plugin for DefaultSceneAnimationPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<DefaultAnimationGraphMap>()
            .add_systems(OnEnter(GameState::InGame), init_scene_root_with_animation.in_set(PostGameInit))
            .add_observer(set_default_animation_after_load)
        ;
    }
}


#[derive(Resource, Default)]
pub struct DefaultAnimationGraphMap {
    map : HashMap<InstanceId, (Handle<AnimationGraph>, AnimationNodeIndex)>
}

impl DefaultAnimationGraphMap {
    pub fn new() -> Self {
        DefaultAnimationGraphMap {
            map: HashMap::new()
        }
    }
    
    pub fn insert(&mut self, instance_id: InstanceId, handle: Handle<AnimationGraph>, node_index: AnimationNodeIndex) {
        self.map.insert(instance_id, (handle, node_index));
    }
    
    pub fn get(&self, instance_id: InstanceId) -> Option<&(Handle<AnimationGraph>, AnimationNodeIndex)> {
        self.map.get(&instance_id)
    }
}

pub fn set_default_animation_after_load(
    trigger: Trigger<SceneInstanceReady>,
    scene_spawner : Res<SceneSpawner>,
    animation_graphs : Res<DefaultAnimationGraphMap>,
    mut instance_root : Query<(Entity, &mut AnimationPlayer)>,
    mut commands : Commands
) {
    let Some((graph, node_index)) = animation_graphs.get(trigger.instance_id) else {
        return;
    };
    
    let Some(entity) = scene_spawner.iter_instance_entities(trigger.instance_id).find(|e| instance_root.contains(*e)) else {
        return;
    };
    instance_root.get_mut(entity).unwrap().1.play(*node_index).repeat();
    
    commands.entity(entity).insert(AnimationGraphHandle(graph.clone()));
}

#[derive(Component)]
pub struct SceneRootWithAnimation {
    pub scene : Handle<Scene>,
    pub animation_graph : Handle<AnimationGraph>,
    pub animation : AnimationNodeIndex,
}

pub fn init_scene_root_with_animation(
    scene_roots : Query<(Entity, &SceneRootWithAnimation)>,
    mut scene_spawner : ResMut<SceneSpawner>,
    mut animation_graphs : ResMut<DefaultAnimationGraphMap>,
) {
    for (entity, scene_root) in scene_roots.iter() {
        let instance_id = scene_spawner.spawn_as_child(scene_root.scene.clone(), entity);
        animation_graphs.insert(instance_id, scene_root.animation_graph.clone(), scene_root.animation);
        println!("Running {instance_id:?}");
    }
}