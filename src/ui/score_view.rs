// Score categories panel widget.
//
// Lists every unlocked category with the score it would produce from the
// current dice. The best available (unused) category is highlighted in yellow.
// Categories already used this room are greyed out.
//
// Example:
//   Chance          18
//   Upper (Fives)   10  *  <- best available, highlighted
//   Full House      25     <- greyed: already used

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Paragraph, Widget},
};

use crate::{
    dice::DicePool,
    scoring::{self, ScoreCategory},
};

pub struct ScoreView<'a> {
    pub pool: &'a DicePool,
    pub unlocked: &'a [ScoreCategory],
    pub used_this_room: &'a [ScoreCategory],
}

impl<'a> ScoreView<'a> {
    pub fn new(
        pool: &'a DicePool,
        unlocked: &'a [ScoreCategory],
        used_this_room: &'a [ScoreCategory],
    ) -> Self {
        Self { pool, unlocked, used_this_room }
    }
}

impl Widget for ScoreView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let values = self.pool.values();

        let best = self
            .unlocked
            .iter()
            .filter(|c| !self.used_this_room.contains(c))
            .max_by_key(|c| scoring::calculate_for(c, &values).unwrap_or(0));

        let lines: Vec<Line> = self
            .unlocked
            .iter()
            .map(|category| {
                let score = scoring::calculate_for(category, &values).unwrap_or(0);
                let used = self.used_this_room.contains(category);
                let is_best = Some(category) == best;

                let style = if used {
                    Style::new().fg(Color::DarkGray)
                } else if is_best {
                    Style::new().fg(Color::Yellow).bold()
                } else {
                    Style::new()
                };

                let marker = if is_best { "*" } else { " " };
                Line::styled(format!("{category:<20} {score:>4} {marker}"), style)
            })
            .collect();

        Paragraph::new(lines).render(area, buf);
    }
}
