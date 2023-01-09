//! A list of [`Component`]s that can be added to an [`Entity`] in our game world.

use specs::prelude::*;
use specs_derive::Component;

/// A component that represents an entity that has a logical (x, y) position in the game world.
#[derive(Component, Clone, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    /// Create a new position component.
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Returns the (f64) distance between this Position and another Position
    pub fn distance(&self, position: &Position) -> f64 {
        (((self.x - position.x).pow(2) + (self.y - position.y).pow(2)) as f64).powf(0.5)
    }

    pub fn after(&self, direction: &Moving) -> Position {
        match direction {
            Moving::Up => Position::new(self.x, self.y - 1),
            Moving::Down => Position::new(self.x, self.y + 1),
            Moving::Left => Position::new(self.x - 1, self.y),
            Moving::Right => Position::new(self.x + 1, self.y),
        }
    }
}

/// An abstract representation of a glyph that can be drawn to the screen to render an entity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glyph {
    Farm,
    Orc,
    Rat,
    Goblin,
    House,
    Player,
    Tree,
    Wall,
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

/// A component that represents an entity that is a non-tree non-player entity.
#[derive(Component, Debug)]
pub struct Town;

/// A component that represents an entity that is moving in a specified direction.
#[derive(Component, Clone, Debug, PartialEq, Eq)]
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
    maximum: u8,
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
        Self {
            amount,
            maximum: amount,
        }
    }

    /// Returns the amount of health.
    #[must_use]
    pub fn amount(&self) -> u8 {
        self.amount
    }

    /// Returns the maximum amount of health.
    #[must_use]
    pub fn maximum(&self) -> u8 {
        self.maximum
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

    /// Increase the maximum amount of health.
    pub fn increase(&mut self, amount: u8) {
        self.maximum = self.maximum.saturating_add(amount);
    }

    /// Increase the amount of health to the maximum.
    pub fn reset(&mut self) {
        self.amount = self.maximum;
    }
}

/// A component that represents an entity that has been defeated.
#[derive(Component, Debug)]
pub struct Defeated;

/// A component that represents an entity that is controlled by the AI.
#[derive(Component, Debug)]
pub enum AI {
    /// The AI will randomly move around the map.
    ///
    /// - If it attempts to move into a friendly, it stops (enforced by combat system).
    /// - If it attempts to move out of bounds, it stops.
    /// - Any other movement is valid (it will attack)
    ///
    /// **STATELESS**: This AI does not store any state.
    Wander,

    /// The AI will move towards the nearest non-tree/non-monster entity.
    ///
    /// - If the player is adjacent, it will attack the player instead.
    /// - It will attack trees if that's the only option.
    ///
    /// **STATELESS**: This AI does not store any state.
    PrioritizeTown,

    /// The AI will move towards the player.
    ///
    /// - If a town is adjacent, it will attack the town instead.
    /// - It will attack trees if that's the only option.
    ///
    /// **STATELESS**: This AI does not store any state.
    PrioritizePlayer,
}
