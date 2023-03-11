/// Use to log messages to the game's console.
#[derive(Debug, Default, Clone)]
pub struct GameLog {
    pub entries: Vec<String>,
}

impl GameLog {
    /// Add an entry to the game log.
    pub fn log<S: ToString>(&mut self, msg: S) {
        let msg = msg.to_string();
        self.entries.push(msg);
    }
}

/// Initialize a new GameLog from a set of messages.
impl From<Vec<String>> for GameLog {
    fn from(entries: Vec<String>) -> Self {
        Self { entries }
    }
}
