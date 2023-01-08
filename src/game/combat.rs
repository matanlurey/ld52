//! Combat system.
//!
//! These systems are responsible for handling combat between entities.

use specs::prelude::*;

use super::{
    components::{Attacking, Defeated, Health, Monster, Moving, Player, Position},
    map::Map,
};

/// A system that converts movement into melee attacks.
///
/// If an entity is moving *into* another entity, it will perform a melee attack.
pub struct ConvertMovementToMeleeAttackSystem;

impl<'a> System<'a> for ConvertMovementToMeleeAttackSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Monster>,
        ReadStorage<'a, Player>,
        ReadExpect<'a, Map>,
        ReadStorage<'a, Health>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, Moving>,
        WriteStorage<'a, Attacking>,
    );

    fn run(&mut self, data: Self::SystemData) {
        // Unpack the system data.
        let (entities, monsters, players, map, health, positions, mut moving, mut attacking) = data;
        let mut stop_movement = Vec::<Entity>::new();

        // Iterate over all entities that have a position and are moving.
        for (entity, position, direction) in (&entities, &positions, &mut moving).join() {
            let mut prospective = position.to_point();

            // Move the entity in the direction it is moving.
            match direction {
                Moving::Up => prospective.y -= 1,
                Moving::Down => prospective.y += 1,
                Moving::Left => prospective.x -= 1,
                Moving::Right => prospective.x += 1,
            }

            // If there would be an overlap with another entity, do not move.
            if let Some(target) = map.get_entity(prospective.x, prospective.y) {
                // Remove the moving component from the entity.
                stop_movement.push(entity);

                // If the target does not have health, do not attack.
                if health.get(target).is_none() {
                    continue;
                }

                // If the entity is a player, only attack monsters.
                let is_player = players.get(entity).is_some();
                if is_player && monsters.get(target).is_none() {
                    continue;
                }

                // Insert an attack.
                attacking.insert(entity, Attacking::new(target)).unwrap();
            }
        }

        // Remove the moving component from the entities that are attacking.
        for entity in stop_movement {
            moving.remove(entity);
        }
    }
}

/// A system that applies attacks, reducing the health of the target.
pub struct ApplyAttackSystem;

impl<'a> System<'a> for ApplyAttackSystem {
    type SystemData = (WriteStorage<'a, Health>, WriteStorage<'a, Attacking>);

    fn run(&mut self, data: Self::SystemData) {
        // Unpack the system data.
        let (mut health, mut attacking) = data;

        // Iterate over all entities that are attacking.
        for attacking in (&mut attacking).join() {
            // Reduce the health of the target.
            let health = health.get_mut(attacking.target()).unwrap();
            health.reduce(1);
        }

        // Remove the attacking component from all entities.
        attacking.clear();
    }
}

/// A system that checks for entities that have been defeated (i.e. have 0 health).
pub struct DefeatSystem;

impl<'a> System<'a> for DefeatSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Health>,
        WriteStorage<'a, Defeated>,
    );

    fn run(&mut self, data: Self::SystemData) {
        // Unpack the system data.
        let (entities, mut health, mut defeated) = data;
        let mut defeated_entities = Vec::<Entity>::new();

        // Iterate over all entities that have health.
        for (entity, health) in (&entities, &health).join() {
            // If the entity has 0 health, remove it.
            if health.amount() == 0 {
                defeated_entities.push(entity);
            }
        }

        // Remove the health component and add the defeated component to all defeated entities.
        for entity in defeated_entities {
            health.remove(entity);
            defeated.insert(entity, Defeated).unwrap();
        }
    }
}

/// A system that removes defeated entities.
pub struct RemoveDefeatedSystem;

impl<'a> System<'a> for RemoveDefeatedSystem {
    type SystemData = (Entities<'a>, ReadStorage<'a, Defeated>);

    fn run(&mut self, data: Self::SystemData) {
        // Unpack the system data.
        let (entities, defeated) = data;
        let mut to_be_removed = Vec::<Entity>::new();

        // Iterate over all entities that have been defeated.
        for (entity, _) in (&entities, &defeated).join() {
            to_be_removed.push(entity);
        }

        // Remove the entities entirely.
        for entity in to_be_removed {
            entities.delete(entity).unwrap();
        }
    }
}