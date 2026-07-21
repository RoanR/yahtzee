// Room types and the data they carry.
//
// A Floor is 3 rooms followed by a boss. Each non-boss room is one of four
// types drawn from a weighted pool. Boss rooms are hardcoded per floor.
//
// Debuffs are active for the entire boss fight and modify how the game loop
// behaves: some constrain the dice pool, others tax HP or alter scoring.

use crate::scoring::ScoreCategory;

// ─── ScoreTarget ──────────────────────────────────────────────────────────────

pub struct ScoreTarget {
    pub required: u32,
    pub reward_gold: u32,
    pub current: u32,
}

// ─── Debuff ───────────────────────────────────────────────────────────────────

pub enum Debuff {
    // One die in the pool always shows 1 at the start of each roll.
    OneDieForcedOne,
    // Each die showing 1 after a roll costs this many extra HP.
    ExtraHpPerOne(u32),
    // Player only gets this many rolls per attempt instead of max_rolls.
    ReducedRolls(u8),
    // One randomly chosen die is locked (cannot be held or rerolled) each roll.
    LockedDie,
    // The score target is doubled; scoring the boss weakness category heals 15 HP.
    DoubleTarget,
}

// ─── BossRoom ─────────────────────────────────────────────────────────────────

pub struct BossRoom {
    pub name: &'static str,
    pub target: ScoreTarget,
    // If the auto-scored result uses this category, it counts for 1.5x progress.
    pub weakness: ScoreCategory,
    pub debuff: Debuff,
}

// ─── Room ─────────────────────────────────────────────────────────────────────

pub enum Room {
    // Standard score challenge: beat target to earn gold, miss to lose HP.
    Challenge(ScoreTarget),
    // Harder optional room: better gold + chance of a rare relic.
    Elite(ScoreTarget),
    // Choose: restore 15 HP or upgrade one die in your pool.
    Rest,
}

impl Room {
    pub fn short_form(&self) -> String {
        match self {
            Room::Challenge(_) => "C".to_string(),
            Room::Elite(_) => "E".to_string(),
            Room::Rest => "R".to_string(),
        }
    }
}
