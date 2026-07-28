// Dungeon header bar widget - single widget for all room types including boss.
//
// Row 0: title (left) | status (right)
// Row 1: target bar (left) | HP bar (right)
//
// Normal rooms:  title = "DUNGEON DICE" / "SUCCESS" / "FAILED" / "REST SITE"
//                status = "Floor X | [room nav] | Gold Xg"
// Boss rooms:    title = "BOSS: <name>"
//                status = "Floor X | Weak: <cat> | Gold: Xg"
//
// Boss mode is detected from GamePhase (Boss or SelectingCategory { from_boss: true }).

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Line,
    text::Span,
    widgets::{Paragraph, Widget},
};

use crate::{
    dungeon::{room::Room, Floor},
    game::{GamePhase, GameState},
};

const BAR_WIDTH: usize = 15;
fn bar(&max: &usize, &current: &usize) -> String {
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
            let chosen = floor.rooms_taken[s];
            (
                floor.room_choices[s][chosen].short_form(),
                Style::new().fg(ratatui::style::Color::DarkGray),
            )
        } else if s == floor.step {
            let text = match floor.current_room() {
                Some(room) => room.short_form(),
                None => "?".to_string(),
            };
            (text, Style::new().fg(ratatui::style::Color::Cyan))
        } else {
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

        let [header_area, second_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);
        let [title_area, status_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(2)]).areas(header_area);
        let [target_area, hp_area] =
            Layout::horizontal([Constraint::Fill(2), Constraint::Fill(2)]).areas(second_area);

        let is_boss = matches!(
            self.state.phase,
            GamePhase::Boss
                | GamePhase::SelectingCategory {
                    from_boss: true,
                    ..
                }
        );

        if is_boss {
            Paragraph::new(format!("BOSS: {}", floor.boss.name)).render(title_area, buf);
            Paragraph::new(format!(
                "Floor {} | Weak: {} | Gold: {}g",
                floor.floor_num, floor.boss.weakness, self.state.gold
            ))
            .right_aligned()
            .render(status_area, buf);
            Paragraph::new(format!(
                "Target: {}",
                bar(
                    &(floor.boss.target.required as usize),
                    &(floor.boss.target.current as usize),
                )
            ))
            .render(target_area, buf);
        } else {
            let status_spans: Vec<Span> = [
                vec![Span::styled(
                    format!("Floor {} |", floor.floor_num),
                    Style::default(),
                )],
                room_nav(floor),
                vec![Span::styled(
                    format!("| Gold {}g", self.state.gold),
                    Style::default(),
                )],
            ]
            .into_iter()
            .flatten()
            .collect();

            let title = match self.state.phase {
                GamePhase::Scored { success, .. } => {
                    if success {
                        "SUCCESS"
                    } else {
                        "FAILED"
                    }
                }
                GamePhase::Rest { .. } => "REST SITE",
                _ => "DUNGEON DICE",
            };

            Paragraph::new(title).render(title_area, buf);
            Paragraph::new(Line::from(status_spans))
                .right_aligned()
                .render(status_area, buf);

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

            if let Some(line) = target_line {
                Paragraph::new(line).render(target_area, buf);
            }
        }

        Paragraph::new(bar(
            &(self.state.max_hp as usize),
            &(self.state.hp as usize),
        ))
        .right_aligned()
        .render(hp_area, buf);
    }
}
