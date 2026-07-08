// Scoring categories and score calculation.
//
// ScoringEngine tracks which categories are unlocked and which have been used
// in the current room. The core calculation logic lives in free functions that
// take only the dice values, keeping them pure and easily testable.
//
// Wild dice (value == WILD sentinel) are substituted for the value that
// maximises the score for each helper independently.
use crate::dice::WILD;
use std::{collections::HashMap, fmt};

// ─── ScoreCategory ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScoreCategory {
    HighDie,
    Chance,
    Ones,
    Twos,
    Threes,
    Fours,
    Fives,
    Sixes,
    FullHouse,
    SmallStraight,
    LargeStraight,
    Yahtzee,
}

impl fmt::Display for ScoreCategory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScoreCategory::HighDie => write!(f, "Highest Die"),
            ScoreCategory::Chance => write!(f, "Chance"),
            ScoreCategory::Ones => write!(f, "Ones"),
            ScoreCategory::Twos => write!(f, "Twos"),
            ScoreCategory::Threes => write!(f, "Threes"),
            ScoreCategory::Fours => write!(f, "Fours"),
            ScoreCategory::Fives => write!(f, "Fives"),
            ScoreCategory::Sixes => write!(f, "Sixes"),
            ScoreCategory::FullHouse => write!(f, "Full house"),
            ScoreCategory::SmallStraight => write!(f, "Small straight"),
            ScoreCategory::LargeStraight => write!(f, "Large straight"),
            ScoreCategory::Yahtzee => write!(f, "Yahtzee"),
        }
    }
}

// ─── Calculation helpers ───────────────────────────────────────────────────────

// Count occurrences of each face value in a slice of dice values.
// Returns an array indexed by face value
fn counts(dice: &[u8]) -> HashMap<u8, usize> {
    let mut counts = HashMap::new();
    for &die in dice {
        *counts.entry(die).or_insert(0) += 1;
    }
    counts
}

// Best upper-section / N-of-a-kind score achievable, assigning all wilds to
// whichever face value produces the highest total.
fn score_counts(dice_sums: &HashMap<u8, usize>, target: u8) -> Option<usize> {
    let bonus_three = 10;
    let bonus_four = 15;

    let count = dice_sums.get(&WILD).unwrap_or(&0) + dice_sums.get(&target).unwrap_or(&0);
    if count >= 4 {
        Some(count * target as usize + bonus_four)
    } else if count >= 3 {
        Some(count * target as usize + bonus_three)
    } else {
        Some(count * target as usize)
    }
}

// Takes raw dice so the value range can be derived dynamically — die faces can
// exceed 6 once upgrades are applied.
//
// 3*face_a + 2*face_b is always maximised by putting the larger value in the
// three slot, so two greedy descending passes suffice: find the best three-face
// first, then the best pair-face with the remaining wild budget. Either `.find()`
// returning None propagates through `?` when no full house is achievable.
fn score_fullhouse(dice: &[u8]) -> Option<usize> {
    let bonus = 25;

    let wild_count = dice.iter().filter(|&&v| v == WILD).count();
    let dice_sums = counts(dice);
    // Wilds can act as values above the current non-wild max; ensure the search
    // ceiling covers at least standard d6 faces.
    let search_max = dice
        .iter()
        .filter(|&&v| v != WILD)
        .max()
        .copied()
        .unwrap_or(0)
        .max(6);

    let face_a = (1u8..=search_max)
        .rev()
        .find(|&v| 3usize.saturating_sub(dice_sums.get(&v).copied().unwrap_or(0)) <= wild_count)?;

    let wilds_for_a = 3usize.saturating_sub(dice_sums.get(&face_a).copied().unwrap_or(0));
    let remaining = wild_count - wilds_for_a;

    let face_b = (1u8..=search_max)
        .rev()
        .filter(|&v| v != face_a)
        .find(|&v| 2usize.saturating_sub(dice_sums.get(&v).copied().unwrap_or(0)) <= remaining)?;

    Some(3 * face_a as usize + 2 * face_b as usize + bonus)
}

fn score_yahtzee(dice_sums: &HashMap<u8, usize>) -> Option<usize> {
    let bonus = 100;

    let wild_count = dice_sums.get(&WILD).copied().unwrap_or(0);
    let total_dice: usize = dice_sums.values().sum();
    let mut best: Option<usize> = None;

    for (&die, &count) in dice_sums.iter() {
        if die == WILD {
            continue;
        }
        // All dice show this face (counting wilds as matching) when no other
        // non-wild face is present.
        if count + wild_count >= total_dice {
            let score = total_dice * die as usize + bonus;
            best = Some(best.map_or(score, |b: usize| b.max(score)));
        }
    }

    // All-wilds pool: score as all 6s (highest standard face).
    if best.is_none() && wild_count == total_dice && wild_count > 0 {
        best = Some(total_dice * 6 + bonus);
    }

    best
}

// Returns the score for the best straight of exactly `len` consecutive values,
// using wilds to fill gaps. Searches every valid starting point and returns the
// highest-scoring run found, or None if no straight is reachable.
fn score_straight(dice: &[u8], len: usize, bonus: usize) -> Option<usize> {
    let wild_count = dice.iter().filter(|&&v| v == WILD).count();

    let mut unique: Vec<u8> = dice.iter().filter(|&&v| v != WILD).copied().collect();
    unique.sort_unstable();
    unique.dedup();

    // Wilds can substitute for values above the current non-wild max; ensure the
    // ceiling covers at least standard d6 faces so wilds have room to be useful.
    let effective_max = unique.last().copied().unwrap_or(0).max(6);

    if unique.is_empty() {
        return if wild_count >= len {
            // All wilds: form the highest possible straight.
            let start = (effective_max as usize + 1).saturating_sub(len);
            Some((start..start + len).sum::<usize>() + bonus)
        } else {
            None
        };
    }

    let min_val = *unique.first().unwrap();
    // Lowest start where a wild could fill below min_val, clamped to 1.
    let start_min = min_val.saturating_sub(wild_count as u8).max(1);
    // Highest start where the run still ends at or before effective_max.
    let start_max = effective_max.saturating_sub((len - 1) as u8);

    if start_min > start_max {
        return None;
    }

    let mut best: Option<usize> = None;

    for start in start_min..=start_max {
        let end = start + (len - 1) as u8;
        let mut wilds_used = 0;
        let mut run_sum = 0usize;

        for val in start..=end {
            if unique.binary_search(&val).is_ok() {
                run_sum += val as usize;
            } else {
                wilds_used += 1;
                run_sum += val as usize;
            }
        }

        if wilds_used <= wild_count {
            best = Some(best.map_or(run_sum + bonus, |b| b.max(run_sum + bonus)));
        }
    }

    best
}

// ─── Core scoring ─────────────────────────────────────────────────────────────

// Calculate the maximum score for a given category
pub fn calculate_for(category: &ScoreCategory, dice: &[u8]) -> Option<usize> {
    let c = counts(dice);
    match category {
        ScoreCategory::HighDie => Some(
            dice.iter()
                .filter(|&&v| v != WILD)
                .max()
                .copied()
                .unwrap_or(0) as usize,
        ),
        ScoreCategory::Chance => Some(
            dice.iter()
                .map(|&v| if v == WILD { 6 } else { v as usize })
                .sum(),
        ),
        ScoreCategory::Ones => score_counts(&c, 1),
        ScoreCategory::Twos => score_counts(&c, 2),
        ScoreCategory::Threes => score_counts(&c, 3),
        ScoreCategory::Fours => score_counts(&c, 4),
        ScoreCategory::Fives => score_counts(&c, 5),
        ScoreCategory::Sixes => score_counts(&c, 6),
        ScoreCategory::FullHouse => score_fullhouse(dice),
        ScoreCategory::SmallStraight => score_straight(dice, 4, 30),
        ScoreCategory::LargeStraight => score_straight(dice, 5, 40),
        ScoreCategory::Yahtzee => score_yahtzee(&c),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice::WILD;

    fn score(cat: ScoreCategory, dice: &[u8]) -> Option<usize> {
        calculate_for(&cat, dice)
    }

    // HighDie: max non-wild face; wilds are skipped
    #[test]
    fn test_high_die() {
        assert_eq!(score(ScoreCategory::HighDie, &[3, 1, 5, 2, 4]), Some(5));
        assert_eq!(
            score(ScoreCategory::HighDie, &[WILD, 3, WILD, WILD, WILD]),
            Some(3)
        );
        assert_eq!(
            score(ScoreCategory::HighDie, &[WILD, WILD, WILD, WILD, WILD]),
            Some(0)
        );
    }

    // Chance: sum all dice; wilds count as 6
    #[test]
    fn test_chance() {
        assert_eq!(score(ScoreCategory::Chance, &[1, 2, 3, 4, 5]), Some(15));
        assert_eq!(score(ScoreCategory::Chance, &[WILD, 1, 1, 1, 1]), Some(10));
    }

    // Upper section (Ones-Sixes): count matching faces + bonus at 3/4+; wilds boost count
    #[test]
    fn test_upper_section() {
        // Three 5s: 3 * 5 + bonus_three(10) = 25
        assert_eq!(score(ScoreCategory::Fives, &[5, 5, 5, 1, 2]), Some(25));
        // One 6 + wild: 2 * 6 = 12 (below bonus threshold)
        assert_eq!(score(ScoreCategory::Sixes, &[6, WILD, 1, 2, 3]), Some(12));
        // No matching: 0
        assert_eq!(score(ScoreCategory::Fours, &[1, 2, 3, 5, 6]), Some(0));
    }

    // FullHouse: greedy two-pass; wilds fill slots; impossible yields None
    #[test]
    fn test_fullhouse() {
        assert_eq!(score(ScoreCategory::FullHouse, &[5, 5, 5, 3, 3]), Some(46));
        assert_eq!(
            score(ScoreCategory::FullHouse, &[4, 4, 4, 6, WILD]),
            Some(49)
        );
        assert_eq!(score(ScoreCategory::FullHouse, &[1, 2, 3, 4, 5]), None);
    }

    // Straights: wilds fill gaps; impossible yields None
    #[test]
    fn test_straights() {
        assert_eq!(
            score(ScoreCategory::SmallStraight, &[2, 3, 4, 5, 5]),
            Some(44)
        );
        assert_eq!(
            score(ScoreCategory::LargeStraight, &[1, 2, 3, 4, 5]),
            Some(55)
        );
        assert_eq!(
            score(ScoreCategory::SmallStraight, &[3, 5, 6, 1, WILD]),
            Some(48)
        );
        assert_eq!(score(ScoreCategory::SmallStraight, &[1, 1, 6, 6, 6]), None);
    }

    // Yahtzee: all matching; wilds count; all-wilds scores as 6s; mixed yields None
    #[test]
    fn test_yahtzee() {
        assert_eq!(score(ScoreCategory::Yahtzee, &[4, 4, 4, 4, 4]), Some(120));
        assert_eq!(
            score(ScoreCategory::Yahtzee, &[3, 3, 3, 3, WILD]),
            Some(115)
        );
        assert_eq!(
            score(ScoreCategory::Yahtzee, &[WILD, WILD, WILD, WILD, WILD]),
            Some(130)
        );
        assert_eq!(score(ScoreCategory::Yahtzee, &[1, 2, 3, 4, 5]), None);
    }
}
