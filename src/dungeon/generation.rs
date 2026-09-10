// Floor and room generation.
//
// Each floor has exactly 3 non-boss rooms drawn from a weighted pool,
// followed by a hardcoded boss for floors 1-5. Beyond floor 5 the boss
// is generated procedurally.
//
// Score target scaling:
//   base     = floor_num * 10
//   elite    = base * 3 / 2
//   boss     = base * 2
//
// Room weights (3 rooms per floor):
//   55% Challenge, 20% Elite, 25% Rest

use rand::Rng;

use crate::scoring::ScoreCategory;

use super::{
    Floor,
    room::{BossRoom, Debuff, Room, ScoreTarget},
};

// ─── Target scaling ───────────────────────────────────────────────────────────

// Base score target for a given floor number (1-indexed).
fn base_target(floor_num: usize, room_num: usize) -> u32 {
    let ret = floor_num
        .saturating_mul(10)
        .saturating_add(floor_num.saturating_mul(room_num)) as u32;
    if ret > 1 { ret } else { 1 }
}

fn challenge_target(floor_num: usize, room_num: usize) -> ScoreTarget {
    let target = base_target(floor_num, room_num);
    ScoreTarget {
        required: target,
        current: target,
        reward_gold: 25,
    }
}

fn elite_target(floor_num: usize, room_num: usize) -> ScoreTarget {
    let target = base_target(floor_num, room_num).saturating_mul(3 / 2);
    ScoreTarget {
        required: target,
        current: target,
        reward_gold: 50,
    }
}

fn boss_target(floor_num: usize) -> ScoreTarget {
    let target = base_target(floor_num, 5).saturating_mul(2);
    ScoreTarget {
        required: target,
        current: target,
        reward_gold: 0,
    }
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
            weakness: ScoreCategory::Chance,
            debuff: Debuff::OneDieForcedOne,
        },
        2 => BossRoom {
            name: "Stone Golem",
            target,
            weakness: ScoreCategory::Sixes,
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
fn random_room(floor_num: usize, room_num: usize, rng: &mut impl Rng) -> Room {
    match rng.random_range(0..100) {
        55..75 => Room::Elite(elite_target(floor_num, room_num)),
        75..100 => Room::Rest,
        _ => Room::Challenge(challenge_target(floor_num, room_num)),
    }
}

// ─── Floor generation ─────────────────────────────────────────────────────────

// Generate a complete floor: 3 pairs of room options + the floor's boss.
pub fn generate_floor(floor_num: usize, rng: &mut impl Rng) -> Floor {
    let room_choices = (0..5)
        .map(|room_num| {
            [
                random_room(floor_num, room_num, rng),
                random_room(floor_num, room_num, rng),
            ]
        })
        .collect();
    let boss = boss_for_floor(floor_num);
    Floor {
        floor_num,
        room_choices,
        rooms_taken: vec![],
        boss,
        step: 0,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn check_st(&st: &ScoreTarget, req_cur: u32, g: u32) {
        assert_eq!(st.required, req_cur);
        assert_eq!(st.current, req_cur);
        assert_eq!(st.reward_gold, g);
    }

    // Base target is computed correctly
    #[test]
    fn test_base_target() {
        assert_eq!(base_target(1, 2), 12);
        assert_eq!(base_target(1, 1), 11);
        assert_eq!(base_target(1, 0), 10);

        // Should have a minimum of 1
        assert_eq!(base_target(0, 0), 1);

        // Should have a maximum of u32::max
        assert_eq!(base_target(usize::MAX, usize::MAX), u32::MAX);
    }

    // Challenge targets are computed correctly
    #[test]
    fn test_challenge_target() {
        let mut challenge = challenge_target(1, 2);
        check_st(&challenge, base_target(1, 2), 25);
        challenge = challenge_target(usize::MAX, usize::MAX);
        check_st(&challenge, base_target(usize::MAX, usize::MAX), 25);
    }

    // Elite targets are computed correctly
    #[test]
    fn test_elite_target() {
        let mut elite = elite_target(1, 2);
        check_st(&elite, base_target(1, 2).saturating_mul(3 / 2), 50);

        elite = elite_target(usize::MAX, usize::MAX);
        check_st(
            &elite,
            base_target(usize::MAX, usize::MAX).saturating_mul(3 / 2),
            50,
        );
    }

    // Boss targets are computed correctly
    #[test]
    fn test_boss_target() {
        let mut boss = boss_target(1);
        check_st(&boss, base_target(1, 5).saturating_mul(2), 0);
        boss = boss_target(usize::MAX);
        check_st(&boss, base_target(usize::MAX, 5).saturating_mul(2), 0);
    }
}
