// Dice types, rolling mechanics, and die upgrade system.
//
// A Die has a fixed set of faces (the values it can show) and tracks its
// current value and held state. DicePool owns the active dice for a room
// and controls roll/hold flow.

use rand::{Rng, rngs::ThreadRng};

// ─── Constants ────────────────────────────────────────────────────────────────

// Sentinel value stored in current_value when a Wild face is showing.
pub const WILD: u8 = 0;

// ─── DieFace ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct DieFace {
    value: u8,
    enchant: Option<usize>,
}

impl DieFace {
    fn new(value: u8) -> Self {
        Self {
            value,
            enchant: None,
        }
    }

    pub fn get_value(&self) -> u8 {
        self.value
    }
}

// ─── DieType ──────────────────────────────────────────────────────────────────

// Determines which face array a Die is built with and any special roll rules.
#[derive(Debug, Clone, PartialEq)]
enum DieType {
    Standard, // [1, 2, 3, 4, 5, 6]
    Wild,     // [1, 2, 3, 4, 5, WILD] — WILD counts as any value when scoring
    Cursed,   // [1, 1, 1, 2, 5, 6]
    Bones,    // [1, 2, 3, 4, 5, 6, 7, 8] — counts as <=6 for combo checks
}

// ─── DieUpgrade ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum DieUpgrade {
    // Replace the single lowest face value with the current highest face value.
    Reface,
    // When this die shows trigger_face, add bonus_score to the room score.
    // Stored on the die; consulted by the scoring layer at score time.
    Enchant {
        face_index: usize,
        bonus_score: usize,
    },
    // Add 1 to the face at face_index in self.faces.
    Augment {
        face_index: usize,
    },
}

// ─── Die ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Die {
    die_type: DieType,
    faces: Vec<DieFace>,
    pub current_value: DieFace,
    pub held: bool,
    pub selected: bool,
}

impl Default for Die {
    fn default() -> Self {
        Self {
            die_type: DieType::Standard,
            faces: vec![
                DieFace::new(1),
                DieFace::new(2),
                DieFace::new(3),
                DieFace::new(4),
                DieFace::new(5),
                DieFace::new(6),
            ],
            current_value: DieFace::new(1),
            held: false,
            selected: false,
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
            faces: vec![
                DieFace::new(1),
                DieFace::new(2),
                DieFace::new(3),
                DieFace::new(4),
                DieFace::new(5),
                DieFace::new(WILD),
            ],
            ..Default::default()
        }
    }

    pub fn cursed() -> Self {
        Self {
            die_type: DieType::Cursed,
            faces: vec![
                DieFace::new(1),
                DieFace::new(1),
                DieFace::new(1),
                DieFace::new(2),
                DieFace::new(5),
                DieFace::new(6),
            ],
            ..Default::default()
        }
    }

    pub fn bones() -> Self {
        Self {
            die_type: DieType::Bones,
            faces: vec![
                DieFace::new(1),
                DieFace::new(2),
                DieFace::new(3),
                DieFace::new(4),
                DieFace::new(5),
                DieFace::new(6),
                DieFace::new(7),
                DieFace::new(8),
            ],
            ..Default::default()
        }
    }

    // ── Upgrades ──────────────────────────────────────────────────────────────

    // Apply a campfire upgrade to this die. Returns false when the upgrade
    // cannot be applied (e.g. Augment with an out-of-bounds face_index).
    pub fn upgrade(&mut self, upgrade: DieUpgrade) -> bool {
        match upgrade {
            DieUpgrade::Reface => {
                // find index of min face, set it to max face value
                let mut max = u8::MIN;
                let mut min = u8::MAX;
                let mut max_index = 0;
                let mut min_index = 0;
                for (index, face) in self.faces.iter().enumerate() {
                    if face.value != WILD && face.value > max {
                        max_index = index;
                        max = face.value;
                    }
                    if face.value != WILD && face.value < min {
                        min_index = index;
                        min = face.value;
                    }
                }
                self.faces[min_index] = self.faces[max_index];
                true
            }
            DieUpgrade::Enchant {
                face_index,
                bonus_score,
            } => {
                if face_index >= self.faces.len() {
                    false
                } else {
                    self.faces[face_index].enchant = Some(bonus_score);
                    true
                }
            }
            DieUpgrade::Augment { face_index } => {
                if face_index >= self.faces.len() {
                    false
                } else {
                    self.faces[face_index].value += 1;
                    true
                }
            }
        }
    }

    // ── Rolling ───────────────────────────────────────────────────────────────

    // Roll this die: pick a random face, returns the new DieFace.
    pub fn roll(&mut self, rng: &mut ThreadRng) -> &DieFace {
        // pick a random index into self.faces
        let idx = rng.random_range(0..self.faces.len());
        self.current_value = self.faces[idx];
        &self.faces[idx]
    }

    // ── Hold / state ──────────────────────────────────────────────────────────

    fn toggle_hold(&mut self) {
        self.held = !self.held
    }

    // True when the die is currently showing a Wild face.
    fn is_wild(&self) -> bool {
        self.current_value.value == WILD
    }

    // Display string for the TUI ("W" for wild, else the number).
    pub fn display_value(&self) -> String {
        if self.is_wild() {
            return "W".to_string();
        }

        self.current_value.value.to_string()
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
    pub rng: ThreadRng,
}

impl DicePool {
    /// Build the default starting pool: 5 standard d6, 3 rolls.
    pub fn new() -> Self {
        Self {
            dice: (0..5).map(|_| Die::standard()).collect(),
            max_rolls: 3,
            rolls_remaining: 3,
            rng: rand::rng(),
        }
    }

    /// Build a pool from an explicit set of dice.
    pub fn with_dice(dice: Vec<Die>, max_rolls: u8) -> Self {
        Self {
            dice,
            max_rolls,
            rolls_remaining: max_rolls,
            rng: rand::rng(),
        }
    }

    /// Roll all non-held, Returns false if no rolls remain.
    pub fn roll_once(&mut self) -> bool {
        if self.rolls_remaining == 0 {
            return false;
        }

        for die in &mut self.dice {
            if die.held {
                continue;
            }
            let _ = die.roll(&mut self.rng);
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
                if d.die_type == DieType::Bones && d.current_value.value > 6 {
                    return 6;
                }
                d.current_value.value
            })
            .collect()
    }

    pub fn can_roll(&self) -> bool {
        self.rolls_remaining > 0
    }

    // Sum of enchant bonuses across all dice for the current face values.
    // Added to the final room score after calculate() runs.
    pub fn enchant_bonus_total(&self) -> usize {
        self.dice
            .iter()
            .map(|d| d.current_value.enchant.unwrap_or(0))
            .sum()
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

    // Replace the first Standard die with a new die (shop purchase).
    // Returns false if no standard die exists in the pool.
    pub fn replace_first_standard_die(&mut self, replacement: Die) -> bool {
        if let Some(idx) = self
            .dice
            .iter()
            .position(|d| matches!(d.die_type, DieType::Standard))
        {
            self.dice[idx] = replacement;
            true
        } else {
            false
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

    pub fn toggle_selected(&mut self) {
        // Find currently selected dice
        let cur = self.dice.iter().position(|x| x.selected);
        //
        match cur {
            None => (),
            Some(x) => self.toggle_hold(x),
        }
    }

    pub fn next_die(&mut self) {
        // Find currently selected dice
        let cur = self.dice.iter().position(|x| x.selected);

        // Select next dice
        let next = match cur {
            None => 0,
            Some(x) => {
                self.dice[x].selected = false;
                if x + 1 < self.dice.len() { x + 1 } else { 0 }
            }
        };
        self.dice[next].selected = true;
    }

    pub fn prev_die(&mut self) {
        // Find currently selected dice
        let cur = self.dice.iter().position(|x| x.selected);

        // Select next dice
        let next = match cur {
            None => 0,
            Some(x) => {
                self.dice[x].selected = false;
                if x == 0 { self.dice.len() - 1 } else { x - 1 }
            }
        };
        self.dice[next].selected = true;
    }
}
