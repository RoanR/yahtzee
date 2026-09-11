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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoring::ScoreCategory;

    fn test_target() -> room::ScoreTarget {
        room::ScoreTarget {
            required: 10,
            reward_gold: 5,
            current: 10,
        }
    }

    // A 2-step floor for deterministic Floor-navigation tests (no RNG involved).
    fn test_floor() -> Floor {
        Floor {
            floor_num: 1,
            room_choices: vec![
                [Room::Challenge(test_target()), Room::Rest],
                [Room::Rest, Room::Elite(test_target())],
            ],
            rooms_taken: vec![],
            boss: BossRoom {
                name: "Test Boss",
                target: test_target(),
                weakness: ScoreCategory::Chance,
                debuff: room::Debuff::OneDieForcedOne,
            },
            step: 0,
        }
    }

    // current_room()/current_room_mut(): None until a choice is made for the
    // current step, then Some(the chosen option)
    #[test]
    fn test_current_room() {
        let mut floor = test_floor();
        assert!(floor.current_room().is_none());
        assert!(floor.current_room_mut().is_none());

        floor.choose(1);
        assert!(matches!(floor.current_room(), Some(Room::Rest)));
        assert!(matches!(floor.current_room_mut(), Some(Room::Rest)));
    }

    // advance(): increments step and returns true while step <= room_choices.len();
    // false once step has moved past that
    #[test]
    fn test_advance() {
        let mut floor = test_floor();
        assert!(floor.advance()); // 0 -> 1
        assert!(floor.advance()); // 1 -> 2 (== room_choices.len(), boss step)
        assert!(floor.advance()); // 2 -> 3
        assert_eq!(floor.step, 3);

        assert!(!floor.advance());
        assert_eq!(floor.step, 3);
    }

    // boss_next(): true exactly when step == room_choices.len()
    #[test]
    fn test_boss_next() {
        let mut floor = test_floor();
        assert!(!floor.boss_next());

        floor.step = floor.room_choices.len();
        assert!(floor.boss_next());

        floor.step += 1;
        assert!(!floor.boss_next());
    }

    // choose(): appends the chosen index to rooms_taken
    #[test]
    fn test_choose() {
        let mut floor = test_floor();
        floor.choose(0);
        floor.choose(1);
        assert_eq!(floor.rooms_taken, vec![0, 1]);
    }

    // next_options(): Some(pair) for the current step, None once boss is next
    #[test]
    fn test_next_options() {
        let mut floor = test_floor();
        assert!(floor.next_options().is_some());

        floor.step = floor.room_choices.len();
        assert!(floor.next_options().is_none());
    }

    // Dungeon::new(): generates exactly floor 1, nothing further
    #[test]
    fn test_dungeon_new() {
        let mut rng = rand::rng();
        let dungeon = Dungeon::new(&mut rng);

        assert_eq!(dungeon.floors.len(), 1);
        assert_eq!(dungeon.current_floor, 0);
        assert_eq!(dungeon.current_floor().floor_num, 1);
    }

    // current_floor_mut(): mutation is visible through current_floor()
    #[test]
    fn test_current_floor_mut() {
        let mut rng = rand::rng();
        let mut dungeon = Dungeon::new(&mut rng);

        dungeon.current_floor_mut().step = 2;
        assert_eq!(dungeon.current_floor().step, 2);
    }

    // descend(): lazily appends exactly one new floor and advances current_floor
    #[test]
    fn test_descend() {
        let mut rng = rand::rng();
        let mut dungeon = Dungeon::new(&mut rng);

        dungeon.descend(&mut rng);
        assert_eq!(dungeon.floors.len(), 2);
        assert_eq!(dungeon.current_floor, 1);
        assert_eq!(dungeon.current_floor().floor_num, 2);

        dungeon.descend(&mut rng);
        assert_eq!(dungeon.floors.len(), 3);
        assert_eq!(dungeon.current_floor, 2);
        assert_eq!(dungeon.current_floor().floor_num, 3);
    }
}
