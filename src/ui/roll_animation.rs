// Roll animation: cycles displayed die values before snapping to the real result.

use rand::Rng;

use crate::dice::DicePool;

// 18 ticks * 16ms ~= 288ms of visible cycling before snapping to the real result.
const ROLL_ANIM_FRAMES: u8 = 18;

pub struct RollAnimation {
    frames_remaining: u8,
    // Per-die value to display this frame. None = held die (show actual current_value).
    // Always 1-6; no WILD sentinel during animation.
    pub display: Vec<Option<u8>>,
}

impl RollAnimation {
    pub fn new(pool: &DicePool, rng: &mut impl Rng) -> Self {
        Self {
            frames_remaining: ROLL_ANIM_FRAMES,
            display: random_display(pool, rng),
        }
    }

    // Advance by one tick. Returns false once the animation has finished, at
    // which point the caller should drop it so real values render.
    pub fn tick(&mut self, pool: &DicePool, rng: &mut impl Rng) -> bool {
        self.frames_remaining -= 1;
        if self.frames_remaining == 0 {
            return false;
        }
        self.display = random_display(pool, rng);
        true
    }
}

fn random_display(pool: &DicePool, rng: &mut impl Rng) -> Vec<Option<u8>> {
    pool.dice
        .iter()
        .map(|d| {
            if d.held {
                None
            } else {
                Some(rng.random_range(1u8..=6))
            }
        })
        .collect()
}
