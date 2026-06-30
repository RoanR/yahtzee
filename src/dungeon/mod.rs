// Dungeon module: floor structure and progression.
//
// A Dungeon is a sequence of Floors generated on demand as the player
// descends. The current floor index advances when the boss is defeated.
// Floors are not pre-generated; generate_floor is called lazily so the
// player's relic/dice state at the time of generation can inform scaling.

pub mod generation;
pub mod room;

use rand::{Rng, seq::IndexedRandom};
use room::{BossRoom, Room};

use crate::dungeon::generation::generate_floor;

// ─── Floor ────────────────────────────────────────────────────────────────────

pub struct Floor {
    pub floor_num: usize, // 1-indexed
    pub rooms: Vec<Room>, // always 3 non-boss rooms
    pub boss: BossRoom,
    pub current_room: usize, // index into rooms; when == rooms.len(), boss is next
}

impl Floor {
    // Returns the current non-boss room, or None if the boss is next.
    pub fn current_room(&self) -> Option<&Room> {
        if self.current_room < self.rooms.len() {
            Some(&self.rooms[self.current_room])
        } else {
            None
        }
    }

    // Advance to the next room. Returns false if already past the last room.
    pub fn advance(&mut self) -> bool {
        if self.current_room <= self.rooms.len() {
            self.current_room += 1;
            true
        } else {
            false
        }
    }

    // True when the player should enter the boss encounter.
    pub fn boss_next(&self) -> bool {
        self.current_room == self.rooms.len()
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
