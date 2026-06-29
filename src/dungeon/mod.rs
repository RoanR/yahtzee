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

// ─── Floor ────────────────────────────────────────────────────────────────────

pub struct Floor {
    pub floor_num: usize,    // 1-indexed
    pub rooms: Vec<Room>,    // always 3 non-boss rooms
    pub boss: BossRoom,
    pub current_room: usize, // index into rooms; when == rooms.len(), boss is next
}

impl Floor {
    // Returns the current non-boss room, or None if the boss is next.
    pub fn current_room(&self) -> Option<&Room> {
        // if current_room < rooms.len() { Some(&rooms[current_room]) } else { None }
        todo!()
    }

    // Advance to the next room. Returns false if already past the last room.
    pub fn advance(&mut self) -> bool {
        // if current_room <= rooms.len() { current_room += 1; true } else { false }
        todo!()
    }

    // True when the player should enter the boss encounter.
    pub fn boss_next(&self) -> bool {
        // current_room == rooms.len()
        todo!()
    }
}

// ─── Dungeon ──────────────────────────────────────────────────────────────────

pub struct Dungeon {
    pub floors: Vec<Floor>,
    pub current_floor: usize, // 0-indexed into floors
}

impl Dungeon {
    // Start a new dungeon run. Generates only the first floor immediately.
    pub fn new(rng: &mut impl Rng) -> Self {
        // floors = vec![generate_floor(1, rng)]
        // Dungeon { floors, current_floor: 0 }
        todo!()
    }

    // The floor the player is currently on.
    pub fn current_floor(&self) -> &Floor {
        // &floors[current_floor]
        todo!()
    }

    pub fn current_floor_mut(&mut self) -> &mut Floor {
        // &mut floors[current_floor]
        todo!()
    }

    // Called after the boss is defeated. Generates and appends the next floor,
    // then increments current_floor.
    pub fn descend(&mut self, rng: &mut impl Rng) {
        // let next_num = floors.len() + 1
        // floors.push(generate_floor(next_num, rng))
        // current_floor += 1
        todo!()
    }
}
