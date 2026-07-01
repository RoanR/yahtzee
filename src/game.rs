// GameState and run lifecycle.
//
// GameState is the single source of truth for a run: dice pool, HP, gold,
// dungeon position, unlocked categories, and active relics.
//
// GamePhase drives which screen the UI renders and which inputs are valid.
// Transitions are handled by methods on GameState so the UI layer stays thin.

use rand::Rng;

use crate::{
    dice::{DicePool, DieUpgrade},
    dungeon::{
        room::{self},
        Dungeon,
    },
    relics::Relic,
    scoring::{self, ScoreCategory},
};

// ─── GamePhase ────────────────────────────────────────────────────────────────

// Tracks which screen/interaction mode the game is in.
pub enum GamePhase {
    // Player is rolling and holding dice.
    Rolling,
    // All rolls used or player chose to score; show result and advance.
    Scored {
        score: usize,
        target: u32,
        success: bool,
    },
    // Boss fight in progress.
    Boss,
    // Player is in a shop room.
    Shop,
    // Player is at a campfire: pick heal or upgrade.
    Rest,
    // Player just defeated a boss; pick a new category to unlock.
    CategoryUnlock,
    // Run is over.
    GameOver,
}

// ─── GameState ────────────────────────────────────────────────────────────────

pub struct GameState {
    pub dice_pool: DicePool,
    pub hp: u32,
    pub max_hp: u32,
    pub gold: u32,
    pub dungeon: Dungeon,
    // Categories the player has unlocked and can score this run.
    pub unlocked: Vec<ScoreCategory>,
    // Categories already scored this room (cleared at the start of each room).
    pub used_this_room: Vec<ScoreCategory>,
    pub relics: Vec<Box<dyn Relic>>,
    pub phase: GamePhase,
    // Base rolls per room before relic bonuses. Stored separately so that
    // begin_room can recompute max_rolls cleanly rather than accumulating.
    base_rolls: u8,
}

impl GameState {
    // Start a new run with default starting stats.
    pub fn new(rng: &mut impl Rng) -> Self {
        Self {
            dice_pool: DicePool::new(),
            hp: 30,
            max_hp: 30,
            gold: 0,
            dungeon: Dungeon::new(rng),
            unlocked: vec![ScoreCategory::HighDie, ScoreCategory::Chance],
            used_this_room: vec![],
            relics: vec![],
            phase: GamePhase::Rolling,
            base_rolls: 3,
        }
    }

    // ── Room lifecycle ────────────────────────────────────────────────────────

    // Called at the start of every new room: reset dice, clear used categories,
    // apply relic on_room_start effects, apply max_rolls from relic bonuses.
    pub fn begin_room(&mut self) {
        self.used_this_room.clear();

        // Apply on_floor_start hooks from relics
        if self.dungeon.current_floor().current_room == 0 {
            self.relics.iter_mut().for_each(|r| r.on_floor_start());
        }

        // Recompute max_rolls from base each room so relic bonuses don't stack.
        let extra: u8 = self.relics.iter().map(|r| r.extra_rolls()).sum();
        self.dice_pool.max_rolls = self.base_rolls + extra;

        self.dice_pool.reset_for_room();
    }

    // Roll the unheld dice. Fires relic on_roll_start hooks before rolling.
    // Returns false if no rolls remain.
    pub fn roll(&mut self) -> bool {
        let first_roll = self.dice_pool.rolls_remaining == self.dice_pool.max_rolls;
        self.relics
            .iter_mut()
            .for_each(|r| r.on_roll_start(&mut self.dice_pool, first_roll));
        self.dice_pool.roll_once()
    }

    // Score the current dice: pick the best available unlocked category,
    // apply enchant bonuses, compare against the room target.
    // Transitions phase to Scored.
    pub fn score(&mut self) {
        let values = self.dice_pool.values();
        let best_category = self
            .unlocked
            .iter()
            .filter(|&c| !self.used_this_room.contains(c))
            .max_by_key(|&c| scoring::calculate_for(c, &values))
            .unwrap_or(&ScoreCategory::HighDie);
        let score = scoring::calculate_for(&best_category, &values).unwrap_or(0)
            + self.dice_pool.enchant_bonus_total();

        if *best_category != ScoreCategory::HighDie {
            self.used_this_room.push(best_category.clone());
        }

        // compare score vs current room target
        let target = match self.dungeon.current_floor().current_room() {
            Some(room::Room::Challenge(x)) => x.required,
            Some(room::Room::Elite(x)) => x.required,
            None | Some(room::Room::Shop) | Some(room::Room::Rest) => return, //Should be unreachable;
        };

        self.phase = GamePhase::Scored {
            score,
            target,
            success: target as usize <= score,
        }
    }

    // ── HP / gold ─────────────────────────────────────────────────────────────

    // Apply HP loss, running it through all relic on_hp_loss hooks first.
    pub fn take_damage(&mut self, amount: u32) {
        let actual = self
            .relics
            .iter_mut()
            .fold(amount, |hp, r| r.on_hp_loss(hp));
        self.hp = self.hp.saturating_sub(actual);
        if self.hp == 0 {
            self.phase = GamePhase::GameOver
        }
    }

    pub fn heal(&mut self, amount: u32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    pub fn earn_gold(&mut self, amount: u32) {
        self.gold += amount;
    }

    pub fn spend_gold(&mut self, amount: u32) -> bool {
        if self.gold >= amount {
            self.gold -= amount;
            true
        } else {
            false
        }
    }

    // ── Progression ───────────────────────────────────────────────────────────

    // Called after the boss is defeated. Transitions to CategoryUnlock.
    pub fn defeat_boss(&mut self) {
        self.phase = GamePhase::CategoryUnlock;
    }

    // Unlock a new scoring category (called from the category-unlock screen).
    pub fn unlock_category(&mut self, category: ScoreCategory) {
        if !self.unlocked.contains(&category) {
            self.unlocked.push(category)
        }
    }

    // Descend to the next floor after unlocking a category.
    pub fn descend(&mut self, rng: &mut impl Rng) {
        self.dungeon.descend(rng);
        self.begin_room();
    }

    // ── Relic management ──────────────────────────────────────────────────────

    // Acquire a relic: add it to the list and apply one-time stat modifiers.
    pub fn acquire_relic(&mut self, relic: Box<dyn Relic>) {
        self.max_hp = (self.max_hp as i32 + relic.max_hp_modifier()).max(1) as u32;
        self.hp = self.hp.min(self.max_hp);
        self.relics.push(relic);
    }

    // ── Die management ────────────────────────────────────────────────────────

    // Apply an upgrade to a die at index in the pool (campfire interaction).
    pub fn upgrade_die(&mut self, die_index: usize, upgrade: DieUpgrade) -> bool {
        self.dice_pool
            .dice
            .get_mut(die_index)
            .map_or_else(|| false, |d| d.upgrade(upgrade))
    }
}
