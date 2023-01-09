use bracket_lib::random::RandomNumberGenerator;
use specs::prelude::*;

use super::{
    components::{Health, Renderable},
    Glyph,
};

pub struct TreeGrowthSystem;

impl<'a> System<'a> for TreeGrowthSystem {
    type SystemData = (
        WriteStorage<'a, Health>,
        ReadStorage<'a, Renderable>,
        WriteExpect<'a, RandomNumberGenerator>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut healths, renderables, mut rng) = data;

        for (health, renderable) in (&mut healths, &renderables).join() {
            // If it's not a tree, do nothing.
            if renderable.glyph() != Glyph::Tree {
                continue;
            }

            // 20% chance to grow.
            if rng.range(0, 100) < 20 {
                health.increase(1);
            }
        }
    }
}
