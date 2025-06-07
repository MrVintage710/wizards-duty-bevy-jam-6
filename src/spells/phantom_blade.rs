use crate::{assets::SpellAssets, enemy::Enemy, spells::{damage::{apply_spell_damage, DestroyOnSpellDamage}, SpellDamage}, util::{player_spell_layer, GameCollisionLayer}};

use super::Spell;
use avian3d::prelude::{Collider, CollisionEventsEnabled, CollisionLayers, CollisionStarted, OnCollisionStart, RigidBody, Sensor};
use bevy::{prelude::*, transform};

pub const PHANTOM_BLADE_COOLDOWN: f32 = 0.3;
pub const PHANTOM_BLADE_DAMAGE: f32 = 3.0;

//==============================================================================================
//        Phantom Blade PLguin
//==============================================================================================

pub struct PhantomBladePlugin;

impl Plugin for PhantomBladePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, phantom_blade_spell_effect)
        ;
    }
}

//==============================================================================================
//        Phantom Blade Spell
//==============================================================================================

pub struct PhantomBlade;

impl Spell for PhantomBlade {
    fn cast(&self, mut commands : Commands, transform : Transform, spell_assets : &SpellAssets) {
        commands.spawn((
            transform,
            InheritedVisibility::default(),
            PhantomBladeSpellEffect::default(),
            children![
                (
                    Transform::from_rotation(Quat::from_rotation_x(-90.0_f32.to_radians())),
                    Sensor,
                    Collider::cylinder(0.1, 1.0),
                    InheritedVisibility::default(),
                    CollisionEventsEnabled,
                    Observer::new(apply_spell_damage),
                    SpellDamage(PHANTOM_BLADE_DAMAGE),
                    DestroyOnSpellDamage,
                    player_spell_layer(),
                    children![
                        (
                            Transform::from_translation(Vec3::new(0.0, -0.5, 0.0)),
                            SceneRoot(spell_assets.dagger.clone()),
                        )
                    ],
                )
            ]
        ));
    }

    fn cooldown(&self) -> f32 {
        PHANTOM_BLADE_COOLDOWN
    }
}

//==============================================================================================
//        PhantomBladeSpellEffect
//==============================================================================================

#[derive(Component)]
#[require(Transform)]
pub struct PhantomBladeSpellEffect(Timer);

impl Default for PhantomBladeSpellEffect {
    fn default() -> Self {
        PhantomBladeSpellEffect(Timer::from_seconds(0.5, TimerMode::Once))
    }
}

pub fn phantom_blade_spell_effect(
    mut commands : Commands,
    mut daggers : Query<(Entity, &mut Transform, &mut PhantomBladeSpellEffect), With<PhantomBladeSpellEffect>>,
    time : Res<Time>
) {
    for (entity, mut transform, mut effect) in daggers.iter_mut() {
        let move_vector = transform.forward() * 20.0 * time.delta_secs();
        transform.translation += move_vector;
        
        effect.0.tick(time.delta());
        if effect.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}

