// GameState and run lifecycle.
//
// GameState is the single source of truth for a run: dice pool, HP, gold,
// dungeon position, unlocked categories, and active relics.
//
// GamePhase drives which screen the UI renders and which inputs are valid.
// Transitions are handled by methods on GameState so the UI layer stays thin.

use rand::Rng;

use crate::{
    dice::{DicePool, Die, DieUpgrade},
    dungeon::{
        Dungeon,
        room::{self},
    },
    relics::Relic,
    scoring::{self, ScoreCategory},
    shop::{ShopItem, ShopItemKind},
};

// ─── UpgradeKind ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub enum UpgradeKind {
    Augment,
    Enchant,
}

// ─── GamePhase ────────────────────────────────────────────────────────────────

// Tracks which screen/interaction mode the game is in.
pub enum GamePhase {
    // Player is rolling and holding dice.
    Rolling,
    // Player is choosing which category to score.
    SelectingCategory {
        cursor: usize,
        from_boss: bool,
    },
    // All rolls used or player chose to score; show result and advance.
    Scored {
        score: usize,
        target: u32,
        success: bool,
    },
    // Boss fight in progress.
    Boss,
    // Player is browsing the shop after a successful fight.
    Shop {
        items: Vec<ShopItem>,
        cursor: usize,
    },
    // Player is at a campfire: pick heal or upgrade.
    Rest,
    // Player is picking which die to upgrade (campfire or shop) or replace (shop special die).
    // When pending_die is Some, Enter replaces the selected die instead of going to face select.
    UpgradeSelectDie { die_cursor: usize, kind: UpgradeKind, from_shop: bool, pending_die: Option<Die> },
    // Player is picking which face of the chosen die to upgrade (campfire or shop).
    UpgradeSelectFace {
        die_index: usize,
        face_cursor: usize,
        kind: UpgradeKind,
        from_shop: bool,
    },
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
        let dungeon = Dungeon::new(rng);
        let starting_phase = match dungeon.current_floor().current_room() {
            Some(room::Room::Rest) => GamePhase::Rest,
            _ => GamePhase::Rolling,
        };
        Self {
            dice_pool: DicePool::new(),
            hp: 30,
            max_hp: 30,
            gold: 0,
            dungeon,
            unlocked: vec![ScoreCategory::HighDie, ScoreCategory::Chance],
            used_this_room: vec![],
            relics: vec![],
            phase: starting_phase,
            base_rolls: 3,
        }
    }

    // ── Room lifecycle ────────────────────────────────────────────────────────

    // Called at the start of every new room: reset dice, clear used categories,
    // apply relic on_room_start effects, apply max_rolls from relic bonuses.
    pub fn begin_room(&mut self, new: bool) {
        if new {
            self.used_this_room.clear();

            // Apply on_floor_start hooks from relics
            if self.dungeon.current_floor().current_room == 0 {
                self.relics.iter_mut().for_each(|r| r.on_floor_start());
            }
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

    // Transition to SelectingCategory so the player can choose which category
    // to score. Cursor starts at 0 (first available category).
    pub fn begin_scoring(&mut self) {
        let from_boss = matches!(self.phase, GamePhase::Boss);
        self.phase = GamePhase::SelectingCategory {
            cursor: 0,
            from_boss,
        };
    }

    // Score the current dice using an explicitly chosen category.
    // Applies enchant bonuses, compares against the room target.
    // Transitions phase to Scored.
    pub fn score_category(&mut self, category: ScoreCategory) {
        let values = self.dice_pool.values();
        let score = scoring::calculate_for(&category, &values).unwrap_or(0)
            + self.dice_pool.enchant_bonus_total();

        let target = match self.dungeon.current_floor_mut().current_room_mut() {
            Some(room::Room::Elite(x)) | Some(room::Room::Challenge(x)) => {
                let ret = x.current;
                x.current = x.current.saturating_sub(score as u32);
                ret
            }
            None => self.dungeon.current_floor().boss.target.current,
            Some(room::Room::Rest) => return,
        };

        if category != ScoreCategory::HighDie {
            self.used_this_room.push(category);
        }

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
        self.begin_room(true);
    }

    // ── Relic management ──────────────────────────────────────────────────────

    // Acquire a relic: apply one-time effects, then add it to the list.
    pub fn acquire_relic(&mut self, relic: Box<dyn Relic>) {
        relic.on_acquire(&mut self.dice_pool);
        self.max_hp = (self.max_hp as i32 + relic.max_hp_modifier()).max(1) as u32;
        self.hp = self.hp.min(self.max_hp);
        self.relics.push(relic);
    }

    // ── Shop ──────────────────────────────────────────────────────────────────

    // Attempt to purchase a shop item. Returns false if the player cannot afford it.
    pub fn buy_shop_item(&mut self, item: ShopItem) -> bool {
        if !self.spend_gold(item.price) {
            return false;
        }
        match item.kind {
            ShopItemKind::Relic(relic) => self.acquire_relic(relic),
            ShopItemKind::SpecialDie(kind) => {
                self.dice_pool.replace_first_standard_die(kind.create_die());
            }
            ShopItemKind::HpPotion(amount) => self.heal(amount),
            ShopItemKind::DieUpgrade(_) => {
                unreachable!("handled by UI before reaching buy_shop_item")
            }
        }
        true
    }

    // ── Die management ────────────────────────────────────────────────────────

    // Replace the die at index with a new die (shop purchase of special die).
    pub fn replace_die_at(&mut self, index: usize, die: Die) {
        self.dice_pool.replace_die(index, die);
    }

    // Apply an upgrade to a die at index in the pool (campfire interaction).
    pub fn upgrade_die(&mut self, die_index: usize, upgrade: DieUpgrade) -> bool {
        self.dice_pool
            .dice
            .get_mut(die_index)
            .map_or_else(|| false, |d| d.upgrade(upgrade))
    }
}
