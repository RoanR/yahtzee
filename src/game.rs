// GameState and run lifecycle.
//
// GameState is the single source of truth for a run: dice pool, HP, gold,
// dungeon position, unlocked categories, and active relics.
//
// GamePhase drives which screen the UI renders and which inputs are valid.
// Transitions are handled by methods on GameState so the UI layer stays thin.

use rand::Rng;

use crate::{
    dice::{Die, DicePool, DieUpgrade},
    dungeon::Dungeon,
    relics::Relic,
    scoring::ScoreCategory,
};

// ─── GamePhase ────────────────────────────────────────────────────────────────

// Tracks which screen/interaction mode the game is in.
pub enum GamePhase {
    // Player is rolling and holding dice.
    Rolling,
    // All rolls used or player chose to score; show result and advance.
    Scored { score: usize, target: u32, success: bool },
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
}

impl GameState {
    // Start a new run with default starting stats.
    pub fn new(rng: &mut impl Rng) -> Self {
        // dice_pool = DicePool::new()
        // hp = max_hp = 30
        // gold = 0
        // dungeon = Dungeon::new(rng)
        // unlocked = vec![ScoreCategory::Chance]
        // used_this_room = vec![]
        // relics = vec![]
        // phase = GamePhase::Rolling
        todo!()
    }

    // ── Room lifecycle ────────────────────────────────────────────────────────

    // Called at the start of every new room: reset dice, clear used categories,
    // apply relic on_room_start effects, apply max_rolls from relic bonuses.
    pub fn begin_room(&mut self) {
        // dice_pool.reset_for_room()
        // used_this_room.clear()
        // apply extra_rolls() from each relic to dice_pool.max_rolls
        // fire on_floor_start hooks if this is the first room of a new floor
        todo!()
    }

    // Roll the unheld dice. Fires relic on_roll_start hooks before rolling.
    // Returns false if no rolls remain.
    pub fn roll(&mut self) -> bool {
        // let first_roll = dice_pool.rolls_remaining == dice_pool.max_rolls
        // relics.iter_mut().for_each(|r| r.on_roll_start(&mut dice_pool, first_roll))
        // dice_pool.roll_once()
        todo!()
    }

    // Score the current dice: pick the best available unlocked category,
    // apply enchant bonuses, compare against the room target.
    // Transitions phase to Scored.
    pub fn score(&mut self) {
        // values = dice_pool.values()
        // best_category = unlocked.iter()
        //     .filter(|c| !used_this_room.contains(c))
        //     .max_by_key(|c| scoring::calculate_for(c, &values))
        // score = scoring::calculate_for(best_category, &values) + dice_pool.enchant_bonus_total()
        // used_this_room.push(best_category)
        // compare score vs current room target
        // transition to GamePhase::Scored { score, target, success }
        todo!()
    }

    // ── HP / gold ─────────────────────────────────────────────────────────────

    // Apply HP loss, running it through all relic on_hp_loss hooks first.
    pub fn take_damage(&mut self, amount: u32) {
        // let actual = relics.iter_mut().fold(amount, |hp, r| r.on_hp_loss(hp))
        // hp = hp.saturating_sub(actual)
        // if hp == 0 { phase = GamePhase::GameOver }
        todo!()
    }

    pub fn heal(&mut self, amount: u32) {
        // hp = (hp + amount).min(max_hp)
        todo!()
    }

    pub fn earn_gold(&mut self, amount: u32) {
        // gold += amount
        // gold += relics.iter().map(|r| r.on_score(score, target)).sum()  -- called from score()
        todo!()
    }

    pub fn spend_gold(&mut self, amount: u32) -> bool {
        // if gold >= amount { gold -= amount; true } else { false }
        todo!()
    }

    // ── Progression ───────────────────────────────────────────────────────────

    // Called after the boss is defeated. Transitions to CategoryUnlock.
    pub fn defeat_boss(&mut self) {
        // phase = GamePhase::CategoryUnlock
        todo!()
    }

    // Unlock a new scoring category (called from the category-unlock screen).
    pub fn unlock_category(&mut self, category: ScoreCategory) {
        // if !unlocked.contains(&category) { unlocked.push(category) }
        todo!()
    }

    // Descend to the next floor after unlocking a category.
    pub fn descend(&mut self, rng: &mut impl Rng) {
        // dungeon.descend(rng)
        // begin_room()
        todo!()
    }

    // ── Relic management ──────────────────────────────────────────────────────

    // Acquire a relic: add it to the list and apply one-time stat modifiers.
    pub fn acquire_relic(&mut self, relic: Box<dyn Relic>) {
        // max_hp = (max_hp as i32 + relic.max_hp_modifier()).max(1) as u32
        // hp = hp.min(max_hp)
        // relics.push(relic)
        todo!()
    }

    // ── Die management ────────────────────────────────────────────────────────

    // Apply an upgrade to a die at index in the pool (campfire interaction).
    pub fn upgrade_die(&mut self, die_index: usize, upgrade: DieUpgrade) -> bool {
        // if let Some(die) = dice_pool.dice.get_mut(die_index) { die.upgrade(upgrade) }
        // else { false }
        todo!()
    }
}
