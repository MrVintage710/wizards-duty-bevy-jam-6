use std::marker::PhantomData;

use avian3d::{position, sync};
use bevy::{prelude::*, render::render_resource::ShaderSize};
use phantom_blade::{phantom_blade_spell_effect, PhantomBlade, PHANTOM_BLADE_COOLDOWN};

use crate::assets::SpellAssets;

pub mod phantom_blade;

//==============================================================================================
//        WeaponsPlugin
//==============================================================================================

pub struct SpellPlugin;

impl Plugin for SpellPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<Spellbook>()
            
            .add_observer(cast_spell)
            
            .add_systems(PreUpdate, tick_spellbook_cooldown)
            .add_systems(Update, phantom_blade_spell_effect)
        ;
    }
}

//==============================================================================================
//        WeaponComponent
//==============================================================================================

const NUMBER_OF_SPELLS: usize = 3;

#[derive(Resource)]
pub struct Spellbook {
    pub spells: [Option<Box<dyn Spell + Send + Sync>>; NUMBER_OF_SPELLS],
    pub cooldown: Timer,
}

impl Default for Spellbook {
    fn default() -> Self {
        Self {
            spells: [Some(Box::new(PhantomBlade)), None, None],
            cooldown: Timer::from_seconds(PHANTOM_BLADE_COOLDOWN, TimerMode::Once),
        }
    }
}

pub trait Spell {
    fn cast(&self, commands : Commands, transform : Transform, spell_assets : &SpellAssets);
    
    fn cooldown(&self) -> f32;
}

//==============================================================================================
//        Spell Systems
//==============================================================================================

#[derive(Event)]
pub struct CastSpell{
    pub position : Vec3,
    pub direction : Vec2,
    pub spell_index : usize
}

fn cast_spell(
    trigger : Trigger<CastSpell>,
    commands : Commands,
    spell_assets : Res<SpellAssets>,
    mut spellbook : ResMut<Spellbook>,
    mut gizmos : Gizmos
) {
    let Some(Some(spell)) = spellbook.spells.get(trigger.spell_index) else {
        return;
    };
    let direction = trigger.direction;
    // gizmos.ray(trigger.position, Vec3::new(direction.x, 0.0, direction.y), Color::srgb(1.0, 0.0, 0.0));
    let angle = direction.y.atan2(direction.x);
    info!("Angle: {}", angle.to_degrees());
    let transform = Transform::from_translation(trigger.position).with_rotation(Quat::from_rotation_y(angle));
    spell.cast(commands, transform, spell_assets.as_ref());
    spellbook.cooldown.reset();
}

fn tick_spellbook_cooldown(
    mut spellbook : ResMut<Spellbook>,
    time : Res<Time>
) {
    spellbook.cooldown.tick(time.delta());
}
