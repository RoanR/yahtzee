// Dungeon header bar widget.
//
// Row 0: "DUNGEON DICE" (left) | "Floor 2 | Room 1/3 | Gold: 50g" (right)
// Row 1: "Target: 80 pts" or "" (left) | HP bar + fraction (right)
//
// HP is moved off the status line and onto row 1 as a visual bar:
//   [========-----]  20 / 30
//   |<--- filled -->|<empty>|
//   filled = hp * BAR_WIDTH / max_hp  (integer, clamped to [0, BAR_WIDTH])
//   bar char: "=" for filled, "-" for empty
//
// Boss header uses the same widget but with different row 0/1 content:
//   Row 0: "BOSS: Rat King" (left) | "Floor 1 | Gold: 50g" (right)
//   Row 1: "Weakness: Chance  Target: 20 pts" (left) | HP bar (right)
//
// Two callers pass different data to achieve this:
//   - DungeonView::new(&state)         for challenge/elite rooms
//   - BossHeaderView::new(&state)      for boss rooms  (new struct, same file)

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Paragraph, Widget},
};

use crate::{
    dungeon::room::Room,
    game::{GamePhase, GameState},
};

// Generate a bar based off a current and max value
const BAR_WIDTH: usize = 15;
fn bar(&max: &usize, &current: &usize) -> String {
    // Make the HP bar and fraction
    let filled = (current * BAR_WIDTH)
        .checked_div(max as usize)
        .unwrap_or(0)
        .min(BAR_WIDTH);
    format!(
        "[{}{}] {} / {}",
        "=".repeat(filled),
        "-".repeat(BAR_WIDTH - filled),
        current,
        max
    )
}

pub struct DungeonView<'a> {
    state: &'a GameState,
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
            "Floor {} | Room {} | Gold: {}g",
            floor.floor_num, room_label, self.state.gold,
        );

        let target_line = match self.state.phase {
            GamePhase::Scored { score, target, .. } => Some(format!("Score {score}/{target}")),
            _ => match floor.current_room() {
                Some(Room::Challenge(t)) => Some(format!(
                    "Target: {}",
                    bar(&(t.required as usize), &(t.current as usize))
                )),
                Some(Room::Elite(t)) => Some(format!(
                    "Elite target: {}",
                    bar(&(t.required as usize), &(t.current as usize))
                )),
                _ => None,
            },
        };

        // Layout: split area into two rows.
        let [header_area, second_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);

        // Row 0: title left, status right.
        let [title_area, status_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(2)]).areas(header_area);

        match self.state.phase {
            GamePhase::Scored { success, .. } => {
                if success {
                    Paragraph::new("SUCCESS").render(title_area, buf)
                } else {
                    Paragraph::new("FAILED").render(title_area, buf)
                }
            }
            _ => Paragraph::new("DUNGEON DICE").render(title_area, buf),
        };
        Paragraph::new(status)
            .right_aligned()
            .render(status_area, buf);

        // Row 1: target left, hp bar right.
        let [target_area, hp_area] =
            Layout::horizontal([Constraint::Fill(2), Constraint::Fill(2)]).areas(second_area);

        if let Some(line) = target_line {
            Paragraph::new(line).render(target_area, buf)
        }
        Paragraph::new(bar(
            &(self.state.max_hp as usize),
            &(self.state.hp as usize),
        ))
        .right_aligned()
        .render(hp_area, buf);
    }
}

// ─── BossHeaderView ───────────────────────────────────────────────────────────

// Separate header widget for the boss screen. Same 2-row layout as DungeonView
// but row 0 shows the boss name and row 1 shows weakness + target.
//
// Row 0: "BOSS: <name>" (left) | "Floor X | Gold: Xg" (right)
// Row 1: "Weakness: <cat>  Target: <n> pts" (left) | HP bar (right)
//
pub struct BossHeaderView<'a> {
    state: &'a GameState,
}

impl<'a> BossHeaderView<'a> {
    pub fn new(state: &'a GameState) -> Self {
        Self { state }
    }
}

impl Widget for BossHeaderView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let floor = self.state.dungeon.current_floor();
        let boss_name = format!("BOSS: {}", floor.boss.name);
        let status = format!("Floor {} | Gold: {}g", floor.floor_num, self.state.gold);
        let weakness = format!(
            "Weakness: {}  Target: {} pts",
            floor.boss.weakness, floor.boss.target.required,
        );

        let [header_area, second_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);

        let [title_area, status_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(2)]).areas(header_area);

        Paragraph::new(boss_name).render(title_area, buf);
        Paragraph::new(status)
            .right_aligned()
            .render(status_area, buf);

        let [weakness_area, hp_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(2)]).areas(second_area);

        Paragraph::new(weakness).render(weakness_area, buf);
        Paragraph::new(bar(
            &(self.state.max_hp as usize),
            &(self.state.hp as usize),
        ))
        .right_aligned()
        .render(hp_area, buf);
    }
}
