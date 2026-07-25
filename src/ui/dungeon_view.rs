// Dungeon header bar widget.
//
// Row 0: "DUNGEON DICE" (left) | "Floor 2 | Room Nav | Gold: 50g" (right)
// Row 1: Target bar + fraction 80 or "" (left) | HP bar + fraction (right)
//
// Boss header uses the same widget but with different row 0/1 content:
//   Row 0: "BOSS: Rat King" (left) | "Floor 1 | Target: 20 pts | Gold: 50g" (right)
//   Row 1: "Weakness: Chance" (left) | HP bar (right)
//
// Two callers pass different data to achieve this:
//   - DungeonView::new(&state)         for challenge/elite rooms
//   - BossHeaderView::new(&state)      for boss rooms  (new struct, same file)

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Line,
    text::Span,
    widgets::{Paragraph, Widget},
};

use crate::{
    dungeon::{Floor, room::Room},
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

// Generate the room navigation display bar
fn room_nav(floor: &Floor) -> Vec<Span<'_>> {
    if floor.boss_next() {
        return vec![Span::styled(
            "Boss",
            Style::new().fg(ratatui::style::Color::Cyan),
        )];
    }

    let total = floor.room_choices.len();
    let mut spans = vec![Span::styled(" [", Style::default())];
    for s in 0..total {
        let (text, style) = if s < floor.step {
            // Completed step: show the chosen room type in gray
            let chosen = floor.rooms_taken[s];
            (floor.room_choices[s][chosen].short_form(), Style::new().fg(ratatui::style::Color::DarkGray))
        } else if s == floor.step {
            // Current step: show room type if chosen, "?" if still choosing
            let text = if floor.rooms_taken.len() > floor.step {
                floor.room_choices[s][floor.rooms_taken[s]].short_form()
            } else {
                "?".to_string()
            };
            (text, Style::new().fg(ratatui::style::Color::Cyan))
        } else {
            // Future step
            ("?".to_string(), Style::default())
        };
        spans.push(Span::styled(text, style));
        if s < total - 1 {
            spans.push(Span::styled("-", Style::default()));
        }
    }
    spans.push(Span::styled("] ", Style::default()));
    spans
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

        // This is an overly complex bit of code to keep the styling on
        // the room display.
        let status_spans: Vec<Span> = vec![
            vec![Span::styled(
                format!("Floor {} |", floor.floor_num),
                Style::default(),
            )],
            room_nav(self.state.dungeon.current_floor()),
            vec![Span::styled(
                format!("| Gold {}g", self.state.gold),
                Style::default(),
            )],
        ]
        .into_iter()
        .flatten()
        .collect();
        let status_line = Line::from(status_spans);

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
            GamePhase::Rest { .. } => Paragraph::new("REST SITE").render(title_area, buf),
            _ => Paragraph::new("DUNGEON DICE").render(title_area, buf),
        };
        Paragraph::new(status_line)
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
// but row 0 shows the boss name and row 1 mirrors the target-bar + HP-bar
// layout used by DungeonView for challenge/elite rooms.
//
// Row 0: "BOSS: <name>" (left) | "Floor X | Weak: <cat> | Gold: Xg" (right)
// Row 1: "Target: <bar>  N / N" (left, Fill(2)) | HP bar (right, Fill(2))
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
        let status = format!(
            "Floor {} | Weak: {} | Gold: {}g",
            floor.floor_num, floor.boss.weakness, self.state.gold,
        );
        let target_line = format!(
            "Target: {}",
            bar(
                &(floor.boss.target.required as usize),
                &(floor.boss.target.current as usize),
            )
        );

        let [header_area, second_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);

        let [title_area, status_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(2)]).areas(header_area);

        Paragraph::new(boss_name).render(title_area, buf);
        Paragraph::new(status)
            .right_aligned()
            .render(status_area, buf);

        let [target_area, hp_area] =
            Layout::horizontal([Constraint::Fill(2), Constraint::Fill(2)]).areas(second_area);

        Paragraph::new(target_line).render(target_area, buf);
        Paragraph::new(bar(
            &(self.state.max_hp as usize),
            &(self.state.hp as usize),
        ))
        .right_aligned()
        .render(hp_area, buf);
    }
}
