use std::{f32::consts::PI, marker::PhantomData};

use avian3d::{position, prelude::OnCollisionStart, sync};
use bevy::{prelude::*, render::render_resource::ShaderSize};
use phantom_blade::{phantom_blade_spell_effect, PhantomBlade, PHANTOM_BLADE_COOLDOWN};

use crate::{assets::SpellAssets, enemy::Enemy};

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
//        Util Structs
//==============================================================================================

#[derive(Component)]
pub struct SpellDamage(u32);

pub fn on_spell_collision(
    trigger : Trigger<OnCollisionStart>,
    mut enemies : Query<&mut Enemy>,
    spells : Query<&SpellDamage>
) -> Result<(), BevyError> {
    let spell = spells.get(trigger.target())?;
    let mut enemy = enemies.get_mut(trigger.collider)?;
    
    enemy.health -= spell.0 as i32;
    Ok(())
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
    let angle = -direction.to_angle();
    let transform = Transform::from_translation(trigger.position).with_rotation(Quat::from_rotation_y(angle - PI / 2.0));
    spell.cast(commands, transform, spell_assets.as_ref());
    spellbook.cooldown.reset();
}

fn tick_spellbook_cooldown(
    mut spellbook : ResMut<Spellbook>,
    time : Res<Time>
) {
    spellbook.cooldown.tick(time.delta());
}
