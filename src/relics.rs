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

use crate::dice::{Die, DicePool};

// ─── Relic trait ──────────────────────────────────────────────────────────────

pub trait Relic {
    fn name(&self) -> &str;
    fn description(&self) -> &str;

    // Called at the start of every roll. first_roll is true only for the first
    // roll of the room (Loaded Dice uses this to reroll 1s once for free).
    fn on_roll_start(&mut self, _pool: &mut DicePool, _first_roll: bool) {}

    // Called when the player would lose hp HP. Returns the actual HP to lose
    // (may be reduced or zeroed). Default: pass through unchanged.
    fn on_hp_loss(&mut self, hp: u32) -> u32 {
        hp
    }

    // Called after scoring. score is the raw score; target is the room target.
    // Returns bonus gold to add on top of the normal reward.
    fn on_score(&self, _score: usize, _target: usize) -> u32 {
        0
    }

    // Called once at the start of each floor; resets per-floor state flags.
    fn on_floor_start(&mut self) {}

    // Flat modifier to max HP (applied once when relic is acquired).
    fn max_hp_modifier(&self) -> i32 {
        0
    }

    // Multiplier applied to shop prices (1.0 = no change, 0.8 = 20% cheaper).
    fn shop_price_multiplier(&self) -> f32 {
        1.0
    }

    // Extra rolls added to DicePool.max_rolls each room.
    fn extra_rolls(&self) -> u8 {
        0
    }

    // Called once when this relic is acquired; lets the relic apply one-time
    // structural changes to the dice pool (e.g. adding a die slot).
    fn on_acquire(&self, _pool: &mut DicePool) {}
}

// ─── Concrete relics ──────────────────────────────────────────────────────────

// Loaded Dice: first roll each room rerolls any die showing 1 once for free.
struct LoadedDice;

impl Relic for LoadedDice {
    fn name(&self) -> &str {
        "Loaded Dice"
    }
    fn description(&self) -> &str {
        "Your first roll each room rerolls any die showing 1 once for free."
    }

    fn on_roll_start(&mut self, _pool: &mut DicePool, _first_roll: bool) {
        if _first_roll {
            for die in &mut _pool.dice {
                if die.current_value.get_value() == 1 {
                    die.roll(&mut _pool.rng);
                }
            }
        }
    }
}

// Extra Die Slot: adds a 6th die slot when acquired.
struct ExtraDieSlot;

impl Relic for ExtraDieSlot {
    fn name(&self) -> &str {
        "Extra Die Slot"
    }

    fn description(&self) -> &str {
        "Adds a 6th die slot to your pool."
    }

    fn on_acquire(&self, pool: &mut DicePool) {
        pool.add_die(Die::standard());
    }
}

// One More Roll: +1 roll per room.
struct OneMoreRoll;

impl Relic for OneMoreRoll {
    fn name(&self) -> &str {
        "One More Roll"
    }

    fn description(&self) -> &str {
        "Gain one extra roll per room."
    }

    fn extra_rolls(&self) -> u8 {
        1
    }
}

// Lucky Horseshoe: failing a target costs 5 HP instead of 10.
struct LuckyHorseshoe;

impl Relic for LuckyHorseshoe {
    fn name(&self) -> &str {
        "Lucky Horseshoe"
    }
    fn description(&self) -> &str {
        "Failing a target costs 5 HP instead of 10."
    }

    fn on_hp_loss(&mut self, hp: u32) -> u32 {
        hp.min(5)
    }
}

// Goblin's Hoard: earn +15 bonus gold when beating a target by 150%+.
struct GoblinsHoard;

impl Relic for GoblinsHoard {
    fn name(&self) -> &str {
        "Goblin's Hoard"
    }
    fn description(&self) -> &str {
        "Earn +15 bonus gold when you beat the target by 150% or more."
    }

    fn on_score(&self, _score: usize, _target: usize) -> u32 {
        if _score >= _target * 3 / 2 { 15 } else { 0 }
    }
}

// Cursed Chalice: -10 max HP; all shop prices 20% cheaper.
struct CursedChalice;

impl Relic for CursedChalice {
    fn name(&self) -> &str {
        "Cursed Chalice"
    }
    fn description(&self) -> &str {
        "-10 max HP, but all shop prices are 20% cheaper."
    }

    fn max_hp_modifier(&self) -> i32 {
        -10
    }
    fn shop_price_multiplier(&self) -> f32 {
        0.8
    }
}

// Enchanted Quill: once per floor, the best-scoring category fires again even
// if already used this room. Tracked via the ScoringEngine; this relic exposes
// a query method the engine checks rather than a hook.
struct EnchantedQuill {
    used_this_floor: bool,
}

impl EnchantedQuill {
    fn new() -> Self {
        Self {
            used_this_floor: false,
        }
    }

    // Called by ScoringEngine when it would skip a used category; returns true
    // if the quill should allow it to fire anyway (once per floor).
    fn try_use(&mut self) -> bool {
        if self.used_this_floor {
            false
        } else {
            self.used_this_floor = true;
            true
        }
    }
}

impl Relic for EnchantedQuill {
    fn name(&self) -> &str {
        "Enchanted Quill"
    }
    fn description(&self) -> &str {
        "Once per floor, the best category can be scored again even if already used."
    }

    fn on_floor_start(&mut self) {
        self.used_this_floor = false;
    }
}

// Shield of the Ancients: the first HP loss each floor is negated.
struct ShieldOfTheAncients {
    used_this_floor: bool,
}

impl ShieldOfTheAncients {
    fn new() -> Self {
        Self {
            used_this_floor: false,
        }
    }
}

impl Relic for ShieldOfTheAncients {
    fn name(&self) -> &str {
        "Shield of the Ancients"
    }
    fn description(&self) -> &str {
        "The first time you would lose HP each floor, negate the damage."
    }

    fn on_hp_loss(&mut self, hp: u32) -> u32 {
        if self.used_this_floor {
            hp
        } else {
            self.used_this_floor = true;
            0
        }
    }

    fn on_floor_start(&mut self) {
        self.used_this_floor = false;
    }
}

// Wizard's Grimoire: once per floor, preview the next roll before committing.
// The actual preview interaction is driven by the UI layer; this relic just
// tracks availability and exposes try_use().
struct WizardsGrimoire {
    used_this_floor: bool,
}

impl WizardsGrimoire {
    fn new() -> Self {
        Self {
            used_this_floor: false,
        }
    }

    // Returns true (and marks used) if the preview is still available this floor.
    fn try_use(&mut self) -> bool {
        if self.used_this_floor {
            false
        } else {
            self.used_this_floor = true;
            true
        }
    }
}

impl Relic for WizardsGrimoire {
    fn name(&self) -> &str {
        "Wizard's Grimoire"
    }
    fn description(&self) -> &str {
        "Once per floor, preview what your next roll will be before committing."
    }

    fn on_floor_start(&mut self) {
        self.used_this_floor = false;
    }
}

// ─── RelicRegistry ────────────────────────────────────────────────────────────

// Returns one fresh instance of every relic. Callers filter out already-held
// relics (by name) before sampling.
pub fn all_relics() -> Vec<Box<dyn Relic>> {
    vec![
        Box::new(LoadedDice),
        Box::new(ExtraDieSlot),
        Box::new(OneMoreRoll),
        Box::new(LuckyHorseshoe),
        Box::new(GoblinsHoard),
        Box::new(CursedChalice),
        Box::new(EnchantedQuill {
            used_this_floor: false,
        }),
        Box::new(ShieldOfTheAncients {
            used_this_floor: false,
        }),
        Box::new(WizardsGrimoire {
            used_this_floor: false,
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // Minimal relic overriding nothing but the required name()/description(),
    // used to exercise the trait's default hook implementations.
    struct DummyRelic;

    impl Relic for DummyRelic {
        fn name(&self) -> &str {
            "Dummy"
        }
        fn description(&self) -> &str {
            "does nothing"
        }
    }

    // Default hooks: HP loss passes through, no score/HP/price/roll modifiers,
    // and the no-op hooks don't panic.
    #[test]
    fn test_relic_trait_defaults() {
        let mut relic = DummyRelic;
        assert_eq!(relic.on_hp_loss(7), 7);
        assert_eq!(relic.on_score(10, 20), 0);
        assert_eq!(relic.max_hp_modifier(), 0);
        assert_eq!(relic.shop_price_multiplier(), 1.0);
        assert_eq!(relic.extra_rolls(), 0);

        relic.on_floor_start();
        let mut pool = DicePool::new();
        relic.on_roll_start(&mut pool, true);
        relic.on_acquire(&mut pool);
    }

    // OneMoreRoll: flat +1 roll per room
    #[test]
    fn test_one_more_roll() {
        assert_eq!(OneMoreRoll.extra_rolls(), 1);
    }

    // CursedChalice: -10 max HP, 20% cheaper shop prices
    #[test]
    fn test_cursed_chalice() {
        assert_eq!(CursedChalice.max_hp_modifier(), -10);
        assert_eq!(CursedChalice.shop_price_multiplier(), 0.8);
    }

    // LoadedDice: on the first roll, dice showing 1 are rerolled to a valid face;
    // dice not showing 1 are untouched. On a later roll, nothing is rerolled.
    #[test]
    fn test_loaded_dice() {
        let mut relic = LoadedDice;
        let mut pool = DicePool::new();
        // Standard die faces are [1,2,3,4,5,6]; index 0 shows 1, index 3 shows 4.
        pool.dice[0].current_value = pool.dice[0].faces()[0];
        pool.dice[1].current_value = pool.dice[1].faces()[3];

        relic.on_roll_start(&mut pool, true);
        assert!((1..=6).contains(&pool.dice[0].current_value.get_value()));
        assert_eq!(pool.dice[1].current_value.get_value(), 4);

        pool.dice[0].current_value = pool.dice[0].faces()[0];
        relic.on_roll_start(&mut pool, false);
        assert_eq!(pool.dice[0].current_value.get_value(), 1);
    }

    // ExtraDieSlot: adds one Standard d6 to the pool on acquire
    #[test]
    fn test_extra_die_slot() {
        let relic = ExtraDieSlot;
        let mut pool = DicePool::new();
        let before = pool.dice.len();

        relic.on_acquire(&mut pool);

        assert_eq!(pool.dice.len(), before + 1);
        assert_eq!(pool.dice.last().unwrap().label(), "D6");
    }

    // LuckyHorseshoe: HP loss capped at 5
    #[test]
    fn test_lucky_horseshoe() {
        let mut relic = LuckyHorseshoe;
        assert_eq!(relic.on_hp_loss(10), 5);
        assert_eq!(relic.on_hp_loss(3), 3);
    }

    // GoblinsHoard: +15 gold at 150%+ of target, 0 below, boundary is inclusive
    #[test]
    fn test_goblins_hoard() {
        let relic = GoblinsHoard;
        assert_eq!(relic.on_score(29, 20), 0);
        assert_eq!(relic.on_score(30, 20), 15);
        assert_eq!(relic.on_score(31, 20), 15);
    }
}
