// Dice types, rolling mechanics, and die upgrade system.
//
// A Die has a fixed set of faces (the values it can show) and tracks its
// current value and held state. DicePool owns the active dice for a room
// and controls roll/hold flow.

use rand::Rng;

// ─── Constants ────────────────────────────────────────────────────────────────

// Sentinel value stored in current_value when a Wild face is showing.
pub const WILD: u8 = 0;

// ─── DieType ──────────────────────────────────────────────────────────────────

// Determines which face array a Die is built with and any special roll rules.
#[derive(Debug, Clone, PartialEq)]
pub enum DieType {
    Standard, // [1, 2, 3, 4, 5, 6]
    Wild,     // [1, 2, 3, 4, 5, WILD] — WILD counts as any value when scoring
    Cursed,   // [1, 1, 1, 2, 5, 6]
    Bones,    // [1, 2, 3, 4, 5, 6, 7, 8] — counts as <=6 for combo checks
}

// ─── Die ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Die {
    pub die_type: DieType,
    pub faces: Vec<u8>,    // all possible face values for this die
    pub current_value: u8, // value currently showing; WILD if wild face up
    pub held: bool,
}

impl Default for Die {
    fn default() -> Self {
        Self {
            die_type: DieType::Standard,
            faces: vec![1, 2, 3, 4, 5, 6],
            current_value: 1,
            held: false,
        }
    }
}

impl Die {
    // ── Constructors ──────────────────────────────────────────────────────────

    pub fn standard() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn wild() -> Self {
        Self {
            die_type: DieType::Wild,
            faces: vec![1, 2, 3, 4, 5, WILD],
            ..Default::default()
        }
    }

    pub fn cursed() -> Self {
        Self {
            die_type: DieType::Cursed,
            faces: vec![1, 1, 1, 2, 5, 6],
            ..Default::default()
        }
    }

    pub fn bones() -> Self {
        Self {
            die_type: DieType::Bones,
            faces: vec![1, 2, 3, 4, 5, 6, 7, 8],
            ..Default::default()
        }
    }

    // ── Rolling ───────────────────────────────────────────────────────────────

    // Roll this die: pick a random face and apply any die-type side effects.
    // Returns the new current_value (or WILD).
    pub fn roll(&mut self, rng: &mut impl Rng) -> u8 {
        // pick a random index into self.faces
        let idx = rng.random_range(0..self.faces.len());
        self.current_value = self.faces[idx];
        self.current_value
    }

    // ── Hold / state ──────────────────────────────────────────────────────────

    pub fn toggle_hold(&mut self) {
        self.held = !self.held
    }

    // True when the die is currently showing a Wild face.
    pub fn is_wild(&self) -> bool {
        self.current_value == WILD
    }

    // Display string for the TUI ("W" for wild, else the number).
    pub fn display_value(&self) -> String {
        if self.is_wild() {
            return "W".to_string();
        }

        self.current_value.to_string()
    }

    // Short human-readable label for the die type (used in TUI dice-type row).
    pub fn label(&self) -> &str {
        match self.die_type {
            DieType::Standard => "D6",
            DieType::Wild => "W6",
            DieType::Cursed => "C6",
            DieType::Bones => "D8",
        }
    }
}

// ─── DicePool ─────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct DicePool {
    pub dice: Vec<Die>,
    pub rolls_remaining: u8,
    pub max_rolls: u8,
}

impl DicePool {
    /// Build the default starting pool: 5 standard d6, 3 rolls.
    pub fn new() -> Self {
        Self {
            dice: (0..5).map(|_| Die::standard()).collect(),
            max_rolls: 3,
            rolls_remaining: 3,
        }
    }

    /// Build a pool from an explicit set of dice.
    pub fn with_dice(dice: Vec<Die>, max_rolls: u8) -> Self {
        Self {
            dice,
            max_rolls,
            rolls_remaining: max_rolls,
        }
    }

    /// Roll all non-held, Returns false if no rolls remain.
    pub fn roll_once(&mut self, rng: &mut impl Rng) -> bool {
        if self.rolls_remaining == 0 {
            return false;
        }

        for die in &mut self.dice {
            if die.held {
                continue;
            }
            let _ = die.roll(rng);
        }
        self.rolls_remaining -= 1;
        true
    }

    // Reset state at the start of a new room.
    pub fn reset_for_room(&mut self) {
        self.rolls_remaining = self.max_rolls;
        // Can't use toggle function as no garantee dice are in hold state
        for die in &mut self.dice {
            die.held = false;
        }
    }

    /// Toggle held state on die at index. No-op if index out of bounds.
    pub fn toggle_hold(&mut self, index: usize) {
        if index < self.dice.len() {
            self.dice[index].toggle_hold();
        }
    }

    // Raw face values for every die, in pool order.
    // Wild dice contribute WILD (0);
    // DragonBone dice over 6 are clamped to 6 here for scoring purposes.
    pub fn values(&self) -> Vec<u8> {
        self.dice
            .iter()
            .map(|d| {
                if d.die_type == DieType::Bones && d.current_value > 6 {
                    return 6;
                }
                d.current_value
            })
            .collect()
    }

    pub fn can_roll(&self) -> bool {
        self.rolls_remaining > 0
    }

    // Add a new die to the pool (e.g. Extra Die Slot relic).
    pub fn add_die(&mut self, die: Die) {
        self.dice.push(die)
    }

    // Remove and return a die at index (e.g. player swapping dice in shop).
    pub fn remove_die(&mut self, index: usize) -> Option<Die> {
        if index < self.dice.len() {
            Some(self.dice.remove(index))
        } else {
            None
        }
    }

    // Replace a die at index with a new die (e.g. buying a special die in shop).
    pub fn replace_die(&mut self, index: usize, die: Die) -> Option<Die> {
        // bounds-check
        if index >= self.dice.len() {
            return None;
        }

        let old = self.dice[index].clone();
        self.dice[index] = die;
        Some(old)
    }
}
