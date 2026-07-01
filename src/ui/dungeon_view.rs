// Dungeon header bar widget.
//
// Renders a single status line across the top of the screen:
//   DUNGEON DICE    Floor 2 | Room 1/3 | HP: 20/30 | Gold: 50g
//
// Also shows the current room type and score target when in a challenge room.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::Widget,
};

use crate::game::GameState;

pub struct DungeonView<'a> {
    pub state: &'a GameState,
}

impl<'a> DungeonView<'a> {
    pub fn new(state: &'a GameState) -> Self {
        Self { state }
    }
}

impl Widget for DungeonView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // floor = state.dungeon.current_floor()
        // room_index = floor.current_room (1-indexed for display)
        // render left: "DUNGEON DICE"
        // render right: "Floor {floor_num} | Room {room_index}/3 | HP: {hp}/{max_hp} | Gold: {gold}g"
        // if floor has a current non-boss room, render the target score on the next line
        todo!()
    }
}
