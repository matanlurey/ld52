//! Turn logs.

use super::Glyph;

/// A message to be displayed in the log.
#[derive(Clone, Debug)]
pub enum LogMessage {
    /// Something was attacked.
    Attacked {
        /// Who attacked.
        attacker: Glyph,

        /// What was attacked.
        target: Glyph,

        /// Where (the target) was attacked, e.g. to highlight optionally.
        position: (i32, i32),

        /// Whether the target was defeated by the attack.
        defeated: bool,
    },
}

/// A singleton that stores logs of events.
pub struct Logs {
    messages: Vec<LogMessage>,
}

impl Logs {
    /// Create a new log.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    /// Add a message to the log.
    pub fn add(&mut self, message: LogMessage) {
        self.messages.push(message);
    }

    /// Returns the log of messages and clears it.
    pub fn flush(&mut self) -> Vec<LogMessage> {
        let messages = self.messages.clone();
        self.messages.clear();
        messages
    }
}
