// Dungeon module: floor structure and progression.
//
// A Dungeon is a sequence of Floors generated on demand as the player
// descends. The current floor index advances when the boss is defeated.
// Floors are not pre-generated; generate_floor is called lazily so the
// player's relic/dice state at the time of generation can inform scaling.

pub mod generation;
pub mod room;

use rand::Rng;
use room::{BossRoom, Room};

use crate::dungeon::generation::generate_floor;

// ─── Floor ────────────────────────────────────────────────────────────────────

pub struct Floor {
    pub floor_num: usize,
    pub room_choices: Vec<[Room; 2]>, // pre-generated pairs; 3 per floor
    pub rooms_taken: Vec<usize>,      // chosen option index (0 or 1) per completed step
    pub boss: BossRoom,
    pub step: usize, // current step: 0-2 = rooms, 3 = boss
}

impl Floor {
    // Returns the active room for the current step, or None if no choice has been
    // made yet (choosing phase) or the boss is next.
    pub fn current_room(&self) -> Option<&Room> {
        let choice = self.rooms_taken.get(self.step)?;
        self.room_choices.get(self.step)?.get(*choice)
    }

    // Mutable variant of current_room.
    pub fn current_room_mut(&mut self) -> Option<&mut Room> {
        let choice = *self.rooms_taken.get(self.step)?;
        self.room_choices.get_mut(self.step)?.get_mut(choice)
    }

    // Advance to the next step. Returns false if already past the last step.
    pub fn advance(&mut self) -> bool {
        if self.step <= self.room_choices.len() {
            self.step += 1;
            true
        } else {
            false
        }
    }

    // True when the player should enter the boss encounter.
    pub fn boss_next(&self) -> bool {
        self.step == self.room_choices.len()
    }

    // Record the player's room choice for the current step. idx is 0 or 1.
    pub fn choose(&mut self, idx: usize) {
        self.rooms_taken.push(idx);
    }

    // Returns the two room options for the current step, or None if boss is next.
    pub fn next_options(&self) -> Option<&[Room; 2]> {
        self.room_choices.get(self.step)
    }
}

// ─── Dungeon ──────────────────────────────────────────────────────────────────

pub struct Dungeon {
    floors: Vec<Floor>,
    current_floor: usize, // 0-indexed into floors
}

impl Dungeon {
    // Start a new dungeon run. Generates only the first floor immediately.
    pub fn new(rng: &mut impl Rng) -> Self {
        Dungeon {
            floors: vec![generate_floor(1, rng)],
            current_floor: 0,
        }
    }

    // The floor the player is currently on.
    pub fn current_floor(&self) -> &Floor {
        &self.floors[self.current_floor]
    }

    pub fn current_floor_mut(&mut self) -> &mut Floor {
        &mut self.floors[self.current_floor]
    }

    // Called after the boss is defeated. Generates and appends the next floor,
    // then increments current_floor.
    pub fn descend(&mut self, rng: &mut impl Rng) {
        let next_num = self.floors.len() + 1;
        self.floors.push(generate_floor(next_num, rng));
        self.current_floor += 1;
    }
}
