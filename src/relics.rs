// Relics: passive items that persist for an entire run.
//
// The Relic trait defines hooks that the game loop calls at specific moments.
// Each hook receives the mutable game state it is allowed to observe or modify.
// Hooks that don't apply to a relic use the default no-op implementations.
//
// Stateful relics (Shield, Quill) carry their own reset flag and rely on
// the game loop calling on_floor_start each floor to clear it.
//
// RelicRegistry holds all relics that can appear in shops / elite rewards.
// The dungeon generator samples from it when populating rooms.

use crate::dice::DicePool;

// ─── Relic trait ──────────────────────────────────────────────────────────────

pub trait Relic {
    fn name(&self) -> &str;
    fn description(&self) -> &str;

    // Called at the start of every roll. first_roll is true only for the first
    // roll of the room (Loaded Dice uses this to reroll 1s once for free).
    fn on_roll_start(&mut self, _pool: &mut DicePool, _first_roll: bool) {}

    // Called when the player would lose hp HP. Returns the actual HP to lose
    // (may be reduced or zeroed). Default: pass through unchanged.
    fn on_hp_loss(&mut self, hp: u32) -> u32 { hp }

    // Called after scoring. score is the raw score; target is the room target.
    // Returns bonus gold to add on top of the normal reward.
    fn on_score(&self, _score: usize, _target: usize) -> u32 { 0 }

    // Called once at the start of each floor; resets per-floor state flags.
    fn on_floor_start(&mut self) {}

    // Flat modifier to max HP (applied once when relic is acquired).
    fn max_hp_modifier(&self) -> i32 { 0 }

    // Multiplier applied to shop prices (1.0 = no change, 0.8 = 20% cheaper).
    fn shop_price_multiplier(&self) -> f32 { 1.0 }

    // Extra rolls added to DicePool.max_rolls each room.
    fn extra_rolls(&self) -> u8 { 0 }
}

// ─── Concrete relics ──────────────────────────────────────────────────────────

// Loaded Dice: first roll each room rerolls any die showing 1 once for free.
pub struct LoadedDice;

impl Relic for LoadedDice {
    fn name(&self) -> &str { "Loaded Dice" }
    fn description(&self) -> &str { "Your first roll each room rerolls any die showing 1 once for free." }

    fn on_roll_start(&mut self, _pool: &mut DicePool, _first_roll: bool) {
        // if first_roll:
        //   for each die in pool where current_value == 1:
        //     die.roll(rng)  -- needs rng threaded in; signature may need extending
        todo!()
    }
}

// Extra Die Slot: adds a 6th die slot when acquired. Applied once at pickup,
// not through a hook; the shop/pickup code calls pool.add_die directly.
pub struct ExtraDieSlot;

impl Relic for ExtraDieSlot {
    fn name(&self) -> &str { "Extra Die Slot" }
    fn description(&self) -> &str { "Adds a 6th die slot to your pool." }
    // No ongoing hooks; effect is applied once at acquisition.
}

// One More Chance: +1 roll per room.
pub struct OneMoreChance;

impl Relic for OneMoreChance {
    fn name(&self) -> &str { "One More Chance" }
    fn description(&self) -> &str { "Gain one extra roll per room." }

    fn extra_rolls(&self) -> u8 { 1 }
}

// Lucky Horseshoe: failing a target costs 5 HP instead of 10.
pub struct LuckyHorseshoe;

impl Relic for LuckyHorseshoe {
    fn name(&self) -> &str { "Lucky Horseshoe" }
    fn description(&self) -> &str { "Failing a target costs 5 HP instead of 10." }

    fn on_hp_loss(&mut self, hp: u32) -> u32 {
        // cap the loss at 5; if the normal loss is already less than 5, keep it
        // pseudo: hp.min(5)
        todo!()
    }
}

// Goblin's Hoard: earn +15 bonus gold when beating a target by 150%+.
pub struct GoblinsHoard;

impl Relic for GoblinsHoard {
    fn name(&self) -> &str { "Goblin's Hoard" }
    fn description(&self) -> &str { "Earn +15 bonus gold when you beat the target by 150% or more." }

    fn on_score(&self, _score: usize, _target: usize) -> u32 {
        // if score >= target * 3 / 2 { 15 } else { 0 }
        todo!()
    }
}

// Cursed Chalice: -10 max HP; all shop prices 20% cheaper.
pub struct CursedChalice;

impl Relic for CursedChalice {
    fn name(&self) -> &str { "Cursed Chalice" }
    fn description(&self) -> &str { "-10 max HP, but all shop prices are 20% cheaper." }

    fn max_hp_modifier(&self) -> i32 { -10 }
    fn shop_price_multiplier(&self) -> f32 { 0.8 }
}

// Enchanted Quill: once per floor, the best-scoring category fires again even
// if already used this room. Tracked via the ScoringEngine; this relic exposes
// a query method the engine checks rather than a hook.
pub struct EnchantedQuill {
    pub used_this_floor: bool,
}

impl EnchantedQuill {
    pub fn new() -> Self { Self { used_this_floor: false } }

    // Called by ScoringEngine when it would skip a used category; returns true
    // if the quill should allow it to fire anyway (once per floor).
    pub fn try_use(&mut self) -> bool {
        // if used_this_floor { false } else { used_this_floor = true; true }
        todo!()
    }
}

impl Relic for EnchantedQuill {
    fn name(&self) -> &str { "Enchanted Quill" }
    fn description(&self) -> &str { "Once per floor, the best category can be scored again even if already used." }

    fn on_floor_start(&mut self) {
        self.used_this_floor = false;
    }
}

// Shield of the Ancients: the first HP loss each floor is negated.
pub struct ShieldOfTheAncients {
    pub used_this_floor: bool,
}

impl ShieldOfTheAncients {
    pub fn new() -> Self { Self { used_this_floor: false } }
}

impl Relic for ShieldOfTheAncients {
    fn name(&self) -> &str { "Shield of the Ancients" }
    fn description(&self) -> &str { "The first time you would lose HP each floor, negate the damage." }

    fn on_hp_loss(&mut self, hp: u32) -> u32 {
        // if used_this_floor { hp } else { used_this_floor = true; 0 }
        todo!()
    }

    fn on_floor_start(&mut self) {
        self.used_this_floor = false;
    }
}

// Wizard's Grimoire: once per floor, preview the next roll before committing.
// The actual preview interaction is driven by the UI layer; this relic just
// tracks availability and exposes try_use().
pub struct WizardsGrimoire {
    pub used_this_floor: bool,
}

impl WizardsGrimoire {
    pub fn new() -> Self { Self { used_this_floor: false } }

    // Returns true (and marks used) if the preview is still available this floor.
    pub fn try_use(&mut self) -> bool {
        // if used_this_floor { false } else { used_this_floor = true; true }
        todo!()
    }
}

impl Relic for WizardsGrimoire {
    fn name(&self) -> &str { "Wizard's Grimoire" }
    fn description(&self) -> &str { "Once per floor, preview what your next roll will be before committing." }

    fn on_floor_start(&mut self) {
        self.used_this_floor = false;
    }
}

// Dice Hoarder: start each floor with one extra die drawn from a spares pool.
// The spares pool (Vec<Die>) lives on GameState; this hook signals the game
// loop to draw from it. Actual draw logic is in the dungeon/game layer.
pub struct DiceHoarder;

impl Relic for DiceHoarder {
    fn name(&self) -> &str { "Dice Hoarder" }
    fn description(&self) -> &str { "Start each floor with one extra die drawn from your spares." }

    fn on_floor_start(&mut self) {
        // signal to game loop: draw one die from GameState::spare_dice into pool
        // implementation lives in game.rs where spare_dice is accessible
    }
}

// ─── RelicRegistry ────────────────────────────────────────────────────────────

// All relics available to appear in shops and elite rooms.
// Returns a fresh instance of every relic; the dungeon generator samples this
// list (excluding relics the player already holds) when populating rooms.
pub struct RelicRegistry;

impl RelicRegistry {
    pub fn all() -> Vec<Box<dyn Relic>> {
        // return one boxed instance of each concrete relic
        // pseudo: vec![Box::new(LoadedDice), Box::new(OneMoreChance), ...]
        todo!()
    }
}
