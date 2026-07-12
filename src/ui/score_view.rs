// Score categories panel widget.
//
// Option A changes:
//   1. A "Scoring" title is rendered on the first line of the area.
//   2. Categories where calculate_for returns None show "--" instead of "0".
//      This visually distinguishes "impossible with current dice" from
//      "scores zero points" (e.g. Ones when no 1s are showing still shows 0).
//   3. Best-category detection is changed to compare Option<usize> directly
//      so that a None-scoring category is never preferred over a Some(0) one.
//
// Layout within the right panel:
//   Line 0: "Scoring" (title, plain style)
//   Line 1: blank separator
//   Lines 2+: one line per unlocked category
//
// Score display format (right-aligned score field, 4 chars wide):
//   "Chance                21  "   <- Some(21)
//   "Small straight         --  "  <- None (impossible)
//   "Fives                  0  "   <- Some(0)  (no 5s showing, but still valid)

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::{
    dice::DicePool,
    scoring::{self, ScoreCategory},
};

pub struct ScoreView<'a> {
    pool: &'a DicePool,
    unlocked: &'a [ScoreCategory],
    used_this_room: &'a [ScoreCategory],
    cursor: Option<usize>,
}

impl<'a> ScoreView<'a> {
    pub fn new(
        pool: &'a DicePool,
        unlocked: &'a [ScoreCategory],
        used_this_room: &'a [ScoreCategory],
    ) -> Self {
        Self {
            pool,
            unlocked,
            used_this_room,
            cursor: None,
        }
    }

    pub fn with_cursor(mut self, cursor: usize) -> Self {
        self.cursor = Some(cursor);
        self
    }
}

impl Widget for ScoreView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let values = self.pool.values();

        let best = self
            .unlocked
            .iter()
            .filter(|c| !self.used_this_room.contains(c))
            .max_by_key(|c| scoring::calculate_for(c, &values));

        let mut lines: Vec<Line> = vec![Line::from("")];
        let mut available_idx: usize = 0;

        lines.extend(self.unlocked.iter().map(|category| {
            let score_str = match scoring::calculate_for(category, &values) {
                Some(n) => format!("{n}"),
                None => "---".to_string(),
            };

            let used = self.used_this_room.contains(category);
            let is_best = Some(category) == best;
            let is_cursor = if used {
                false
            } else {
                let matches = self.cursor == Some(available_idx);
                available_idx += 1;
                matches
            };

            let style = if used {
                Style::new().fg(Color::DarkGray)
            } else if is_cursor {
                Style::new().fg(Color::Cyan).bold()
            } else if is_best {
                Style::new().fg(Color::Yellow).bold()
            } else {
                Style::new()
            };

            let marker = if is_cursor {
                "<"
            } else if is_best {
                "*"
            } else {
                " "
            };

            Line::styled(format!(" {category:<15} {score_str} {marker}"), style)
        }));

        Paragraph::new(lines)
            .block(
                Block::bordered()
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .title_top(Line::from("Scoring").bold().centered()),
            )
            .render(area, buf);
    }
}
