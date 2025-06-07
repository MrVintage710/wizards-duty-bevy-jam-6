
use avian3d::prelude::OnCollisionStart;
use bevy::prelude::*;

use crate::util::Health;

//==============================================================================================
//        DamageBox Plugin
//==============================================================================================

pub struct DamageBoxPlugin;

impl Plugin for DamageBoxPlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
}

//==============================================================================================
//        SpellDamage Component
//==============================================================================================

#[derive(Component)]
pub struct SpellDamage(pub f32);

#[derive(Component)]
pub struct RemoveOnSpellDamage;

#[derive(Component)]
pub struct DestroyOnSpellDamage;

//==============================================================================================
//        SpellDamage Systems
//==============================================================================================

pub fn apply_spell_damage(
    trigger : Trigger<OnCollisionStart>,
    mut commands : Commands,
    mut target : Query<&mut Health>,
    spells : Query<(Entity, &SpellDamage, Option<&DestroyOnSpellDamage>)>
) -> Result<(), BevyError> {
    let (entity, spell_damage, destroy_on_spell_damage) = spells.get(trigger.target())?;
    let mut enemy = target.get_mut(trigger.collider)?;
    
    enemy.take_damage(spell_damage.0);
    
    if destroy_on_spell_damage.is_some() { commands.entity(entity).despawn(); };
    Ok(())
}

pub fn spawn_damage_box(commands : &mut Commands, ) {
    
}