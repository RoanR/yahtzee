// Floor and room generation.
//
// Each floor has exactly 3 non-boss rooms drawn from a weighted pool,
// followed by a hardcoded boss for floors 1-5. Beyond floor 5 the boss
// is generated procedurally.
//
// Score target scaling:
//   base     = 50 + floor_num * 30
//   elite    = base * 3 / 2
//   boss     = base * 2
//
// Room weights (3 rooms per floor):
//   50% Challenge, 20% Elite, 15% Shop, 15% Rest

use rand::Rng;

use crate::scoring::ScoreCategory;

use super::{Floor, room::{BossRoom, Debuff, Room, ScoreTarget}};

// ─── Target scaling ───────────────────────────────────────────────────────────

// Base score target for a given floor number (1-indexed).
fn base_target(floor_num: usize) -> u32 {
    // 50 + floor_num * 30
    todo!()
}

fn challenge_target(floor_num: usize) -> ScoreTarget {
    // ScoreTarget { required: base_target(floor_num), reward_gold: 25 }
    todo!()
}

fn elite_target(floor_num: usize) -> ScoreTarget {
    // ScoreTarget { required: base_target(floor_num) * 3 / 2, reward_gold: 50 }
    todo!()
}

fn boss_target(floor_num: usize) -> ScoreTarget {
    // ScoreTarget { required: base_target(floor_num) * 2, reward_gold: 0 }
    todo!()
}

// ─── Boss data ────────────────────────────────────────────────────────────────

// Returns the BossRoom for a specific floor. Floors 1-5 are hardcoded;
// beyond that, a procedural boss is generated from floor_num.
fn boss_for_floor(floor_num: usize) -> BossRoom {
    let target = boss_target(floor_num);
    match floor_num {
        1 => BossRoom {
            name: "Rat King",
            target,
            weakness: ScoreCategory::ThreeOfAKind,
            debuff: Debuff::OneDieForcedOne,
        },
        2 => BossRoom {
            name: "Stone Golem",
            target,
            weakness: ScoreCategory::Sixes, // Upper section: best represented by Sixes
            debuff: Debuff::ExtraHpPerOne(2),
        },
        3 => BossRoom {
            name: "Goblin King",
            target,
            weakness: ScoreCategory::FullHouse,
            debuff: Debuff::ReducedRolls(2),
        },
        4 => BossRoom {
            name: "Dark Wizard",
            target,
            weakness: ScoreCategory::SmallStraight,
            debuff: Debuff::LockedDie,
        },
        5 => BossRoom {
            name: "The Dragon",
            target,
            weakness: ScoreCategory::Yahtzee,
            debuff: Debuff::DoubleTarget,
        },
        _ => {
            // procedural boss beyond floor 5
            // pseudo: scale debuff severity with floor_num, pick weakness at random
            todo!()
        }
    }
}

// ─── Room generation ──────────────────────────────────────────────────────────

// Pick a single non-boss room type from the weighted pool.
fn random_room(floor_num: usize, rng: &mut impl Rng) -> Room {
    // draw a value in 0..100 and map to room type by cumulative weight:
    //   0..50  => Challenge
    //   50..70 => Elite
    //   70..85 => Shop
    //   85..100 => Rest
    todo!()
}

// ─── Floor generation ─────────────────────────────────────────────────────────

// Generate a complete floor: 3 random rooms + the floor's boss.
pub fn generate_floor(floor_num: usize, rng: &mut impl Rng) -> Floor {
    // rooms = (0..3).map(|_| random_room(floor_num, rng)).collect()
    // boss  = boss_for_floor(floor_num)
    // Floor { floor_num, rooms, boss }
    todo!()
}
