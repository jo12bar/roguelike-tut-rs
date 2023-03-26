use rltk::RandomNumberGenerator;

/// An entry in a [`RngTable`].
#[derive(Debug, Clone)]
pub(crate) struct RngTableEntry {
    name: String,
    weight: i32,
}

impl RngTableEntry {
    pub fn new<S: ToString>(name: S, weight: i32) -> Self {
        Self {
            name: name.to_string(),
            weight,
        }
    }
}

impl<S: ToString> From<(S, i32)> for RngTableEntry {
    fn from((name, weight): (S, i32)) -> Self {
        Self::new(name, weight)
    }
}

/// A "spawn table" for defining the relative probabilities of random events occurring.
#[derive(Default)]
pub(crate) struct RngTable {
    entries: Vec<RngTableEntry>,
    total_weight: i32,
}

impl RngTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new entry to the table.
    pub fn add<S: ToString>(mut self, name: S, weight: i32) -> Self {
        self.add_entry(RngTableEntry::new(name, weight));
        self
    }

    fn add_entry(&mut self, entry: RngTableEntry) {
        self.total_weight += entry.weight;
        self.entries.push(entry);
    }

    /// Roll the table for some result. The returned string will be an entry
    /// previously added with [`RngTable::add()`].
    ///
    /// If no entries have been added, `None` will be returned.
    /// `None` will also be returned if every roll for every table entry fails.
    pub fn roll<'a>(&'a self, rng: &mut RandomNumberGenerator) -> Option<&'a str> {
        if self.total_weight == 0 {
            return None;
        }

        let mut roll = rng.roll_dice(1, self.total_weight) - 1;
        let mut index = 0;

        while roll > 0 {
            println!("i: {index}, roll: {roll}");
            if roll < self.entries[index].weight {
                return Some(&self.entries[index].name);
            }

            roll -= self.entries[index].weight;
            index += 1;
        }

        None
    }
}

impl From<&[RngTableEntry]> for RngTable {
    fn from(entries: &[RngTableEntry]) -> Self {
        let mut this = Self::new();
        for entry in entries.iter() {
            this.add_entry(entry.clone());
        }
        this
    }
}
