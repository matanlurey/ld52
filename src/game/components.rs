//! A list of [`Component`]s that can be added to an [`Entity`] in our game world.

use bracket_lib::terminal::Point;
use specs::prelude::*;
use specs_derive::Component;

/// A component that represents an entity that has a logical (x, y) position in the game world.
#[derive(Component, Debug)]
pub struct Position {
    x: i32,
    y: i32,
}

impl Position {
    /// Create a new position component.
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Update the position to a new (x, y) coordinate.
    pub fn update(&mut self, to: &Point) {
        self.x = to.x;
        self.y = to.y;
    }

    /// Returns the position as a tuple of (x, y).
    pub fn to_point(&self) -> Point {
        Point::new(self.x, self.y)
    }
}

/// An abstract representation of a glyph that can be drawn to the screen to render an entity.
#[derive(Clone, Copy, Debug)]
pub enum Glyph {
    Goblin,
    Player,
}

/// A component that represents an entity that can be drawn to the screen.
#[derive(Component, Debug)]
pub struct Renderable {
    glyph: Glyph,
}

impl Renderable {
    /// Create a new renderable component.
    pub fn new(glyph: Glyph) -> Self {
        Self { glyph }
    }

    /// Returns the glyph that represents the entity.
    pub fn glyph(&self) -> Glyph {
        self.glyph
    }
}

/// A component that represents an entity that can be controlled by the player.
#[derive(Component, Debug)]
pub struct Player;

/// A component that represents a hostile (monster) entity.
#[derive(Component, Debug)]
pub struct Monster;

/// A component that represents an entity that is moving in a specified direction.
#[derive(Component, Debug)]
pub enum Moving {
    Up,
    Down,
    Left,
    Right,
}

/// A component that represents an entity that is attacking another entity.
#[derive(Component, Debug)]
pub struct Attacking {
    target: Entity,
}

impl Attacking {
    /// Create a new attacking component.
    pub fn new(target: Entity) -> Self {
        Self { target }
    }

    /// Returns the entity that is being attacked.
    pub fn target(&self) -> Entity {
        self.target
    }
}

/// A component that represents an entity that has health.
#[derive(Component, Debug)]
pub struct Health {
    amount: u8,
}

/// The possible states of an entity's health.
pub enum HealthState {
    Alive,
    Defeated,
}

impl Health {
    /// Create a new health component.
    #[must_use]
    pub fn new(amount: u8) -> Self {
        Self { amount }
    }

    /// Returns the amount of health.
    #[must_use]
    pub fn amount(&self) -> u8 {
        self.amount
    }

    /// Decrease the amount of health, returning current state of the entity.
    pub fn reduce(&mut self, amount: u8) -> HealthState {
        self.amount = self.amount.saturating_sub(amount);
        if self.amount == 0 {
            HealthState::Defeated
        } else {
            HealthState::Alive
        }
    }
}

/// A component that represents an entity that has been defeated.
#[derive(Component, Debug)]
pub struct Defeated;
