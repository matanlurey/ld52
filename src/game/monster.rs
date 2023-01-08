use specs::prelude::*;

use super::{
    components::{Monster, Moving},
    map::Map,
    RunState,
};

/// Determines how monsters will react, such as moving around the map.
pub struct MonsterAISystem;

impl<'a> System<'a> for MonsterAISystem {
    type SystemData = (
        ReadExpect<'a, Map>,
        ReadExpect<'a, RunState>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Moving>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (_map, state, monsters, mut moving) = data;

        // If this is not the monster's turn, do nothing.
        if *state != RunState::MonsterTurn {
            return;
        }

        // Iterate over all monsters.
        for (_monster, _moving) in (&monsters, &mut moving).join() {
            // TODO: Implement monster AI.
        }
    }
}
