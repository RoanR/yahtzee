// Score categories panel widget.
//
// Lists every unlocked category with the score it would produce from the
// current dice. The highest available (unused) score is marked with a star.
// Categories already used this room are greyed out.
//
// Example:
//   Chance          18
//   Upper (Fives)   10
//   Full House      25 *
//   Small Straight   0  (greyed: already used)

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Widget,
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
        // values = self.pool.values()
        // for each category in self.unlocked:
        //   score = scoring::calculate(&values)   (filtered to this category)
        //   used  = self.used_this_room.contains(&category)
        //   best  = highest score among non-used categories
        //   style = grey if used, bold if best, normal otherwise
        //   render "{category_name:<20} {score:>4} {star}"
        todo!()
    }
}
