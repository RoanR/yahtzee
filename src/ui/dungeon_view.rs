// Dungeon header bar widget.
//
// Renders two lines at the top of the screen:
//   Line 1: "DUNGEON DICE" (left) | "Floor 2 | Room 1/3 | HP: 20/30 | Gold: 50g" (right)
//   Line 2: "Target: 80 pts" when in a Challenge or Elite room, blank otherwise.

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Paragraph, Widget},
};

use crate::{
    dungeon::room::Room,
    game::GameState,
};

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
        let floor = self.state.dungeon.current_floor();

        let room_label = if floor.boss_next() {
            "Boss".to_string()
        } else {
            format!("{}/{}", floor.current_room + 1, floor.rooms.len())
        };

        let status = format!(
            "Floor {} | Room {} | HP: {}/{} | Gold: {}g",
            floor.floor_num,
            room_label,
            self.state.hp,
            self.state.max_hp,
            self.state.gold,
        );

        let target_line = match floor.current_room() {
            Some(Room::Challenge(t)) => Some(format!("Target: {} pts", t.required)),
            Some(Room::Elite(t)) => Some(format!("Elite target: {} pts", t.required)),
            _ => None,
        };

        // Split into a status row and an optional target row.
        let [header_area, target_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(area);

        // Split the header row into title (left) and status (right).
        let [title_area, status_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(2),
        ])
        .areas(header_area);

        Paragraph::new("DUNGEON DICE").render(title_area, buf);
        Paragraph::new(status).right_aligned().render(status_area, buf);

        if let Some(line) = target_line {
            Paragraph::new(line).render(target_area, buf);
        }
    }
}
