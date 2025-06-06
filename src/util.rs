
use std::collections::HashMap;

use avian3d::prelude::{CollisionLayers, PhysicsLayer};
use bevy::{ecs::{entity, schedule::ScheduleLabel}, prelude::*, scene::{InstanceId, SceneInstanceReady}, state::commands};

use crate::{enemy::minion::minion_stabbed, GameState};

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

pub fn player_spell_layer() -> CollisionLayers {
    CollisionLayers::new(GameCollisionLayer::Spell, GameCollisionLayer::Enemy)
}

pub fn enemy_spell_layer() -> CollisionLayers {
    CollisionLayers::new(GameCollisionLayer::Spell, GameCollisionLayer::Player)
}

//==============================================================================================
//        General Purpose Components
//==============================================================================================

#[derive(Component)]
pub struct Health {
    pub max_health: u32,
    pub current_health: u32,
}

impl Health {
    pub fn new(max_health: u32) -> Self {
        Health {
            max_health,
            current_health: max_health,
        }
    }
    
    pub fn take_damage(&mut self, damage: u32) {
        self.current_health = self.current_health.saturating_sub(damage);
    }
    
    pub fn heal(&mut self, amount: u32) {
        self.current_health += amount;
        if self.current_health > self.max_health {
            self.current_health = self.max_health;
        }
    }
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
    map : HashMap<InstanceId, (SceneRootWithAnimation, Entity)>
}

impl DefaultAnimationGraphMap {
    pub fn new() -> Self {
        DefaultAnimationGraphMap {
            map: HashMap::new()
        }
    }
    
    pub fn insert(&mut self, instance_id: InstanceId, scene_root: SceneRootWithAnimation, entity: Entity) {
        self.map.insert(instance_id, (scene_root, entity));
    }
    
    pub fn get(&self, instance_id: InstanceId) -> Option<&(SceneRootWithAnimation, Entity)> {
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
    let Some((scene_root, controler_entity)) = animation_graphs.get(trigger.instance_id) else {
        return;
    };
    
    let Some(entity) = scene_spawner.iter_instance_entities(trigger.instance_id).find(|e| instance_root.contains(*e)) else {
        return;
    };
    for index in scene_root.animations.iter() {
        let (_, mut animation_player) = instance_root.get_mut(entity).unwrap();
        let options = animation_player.play(*index);
        if scene_root.repeating {
            options.repeat();
        }
    }
    
    commands.entity(entity).insert((AnimationGraphHandle(scene_root.animation_graph.clone()), AnimatedModelFor(*controler_entity)));
    if let Some(transform) = scene_root.transform {
        commands.entity(entity).insert(transform);
    }
    commands.trigger_targets(AnimatedSceneCreated(entity), *controler_entity);
}

#[derive(Component, Clone)]
pub struct SceneRootWithAnimation {
    pub scene : Handle<Scene>,
    pub animation_graph : Handle<AnimationGraph>,
    pub animations : Vec<AnimationNodeIndex>,
    pub repeating : bool,
    pub transform : Option<Transform>,
}

impl SceneRootWithAnimation {
    pub fn new(scene: Handle<Scene>, animation_graph: Handle<AnimationGraph>) -> Self {
        Self {
            scene,
            animation_graph,
            animations: Vec::new(),
            repeating: false,
            transform: None,
        }
    }
    
    pub fn with_animation(mut self, animation: AnimationNodeIndex) -> Self {
        self.animations.push(animation);
        self
    }
    
    pub fn repeat(mut self) -> Self {
        self.repeating = true;
        self
    }
    
    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = Some(transform);
        self
    }
}

pub fn init_scene_root_with_animation(
    scene_roots : Query<(Entity, &SceneRootWithAnimation)>,
    mut scene_spawner : ResMut<SceneSpawner>,
    mut animation_graphs : ResMut<DefaultAnimationGraphMap>,
) {
    for (entity, scene_root) in scene_roots.iter() {
        let instance_id = scene_spawner.spawn_as_child(scene_root.scene.clone(), entity);
        animation_graphs.insert(instance_id, scene_root.clone(), entity);
        println!("Running {instance_id:?}");
    }
}

#[derive(Event, Clone)]
pub struct AnimatedSceneCreated(pub Entity);

/// This is a "relationship" component.
/// Add it to an entity that "likes" another entity.
#[derive(Component)]
#[relationship(relationship_target = AnimationControlerFor)]
struct AnimatedModelFor(pub Entity);

/// This is the "relationship target" component.
/// It will be automatically inserted and updated to contain
/// all entities that currently "like" this entity.
#[derive(Component, Deref)]
#[relationship_target(relationship = AnimatedModelFor)]
struct AnimationControlerFor(Vec<Entity>);