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
    Chance,
    Ones,
    Twos,
    Threes,
    Fours,
    Fives,
    Sixes,
    ThreeOfAKind,
    FourOfAKind,
    FullHouse,
    SmallStraight,
    LargeStraight,
    Yahtzee,
}

impl fmt::Display for ScoreCategory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScoreCategory::Chance => write!(f, "Chance"),
            ScoreCategory::Ones => write!(f, "Ones"),
            ScoreCategory::Twos => write!(f, "Twos"),
            ScoreCategory::Threes => write!(f, "Threes"),
            ScoreCategory::Fours => write!(f, "Fours"),
            ScoreCategory::Fives => write!(f, "Fives"),
            ScoreCategory::Sixes => write!(f, "Sixes"),
            ScoreCategory::ThreeOfAKind => write!(f, "Three of a kind"),
            ScoreCategory::FourOfAKind => write!(f, "Four of a kind"),
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
fn score_counts(dice_sums: &HashMap<u8, usize>) -> Option<usize> {
    let bonus_three = 10;
    let bonus_four = 15;

    let wild_count = dice_sums.get(&WILD).copied().unwrap_or(0);
    let mut max = 0;

    for (&die, &count) in dice_sums.iter() {
        if die == WILD {
            continue;
        }
        let effective = count + wild_count;
        let bonus = match effective {
            3 => bonus_three,
            e if e >= 4 => bonus_four,
            _ => 0,
        };
        max = max.max(die as usize * effective + bonus);
    }
    Some(max)
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
    let search_max = dice.iter().filter(|&&v| v != WILD).max().copied().unwrap_or(0).max(6);

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

// Calculate the maximum score that can be made from a slice of die values.
pub fn calculate(dice: &[u8]) -> usize {
    let mut max = 0;
    let c = counts(dice);

    // Chance
    max = max.max(
        dice.iter()
            .map(|&v| if v == WILD { 6 } else { v as usize })
            .sum(),
    );
    // Upper
    max = max.max(score_counts(&c).unwrap_or(0));
    // full house
    max = max.max(score_fullhouse(dice).unwrap_or(0));
    // SmallStraight:
    max = max.max(score_straight(dice, 4, 30).unwrap_or(0));
    // LargeStraight:
    max = max.max(score_straight(dice, 5, 40).unwrap_or(0));
    // Yahtzee:
    max = max.max(score_yahtzee(&c).unwrap_or(0));
    max
}
