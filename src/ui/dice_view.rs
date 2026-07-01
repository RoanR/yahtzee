// Dice display widget.
//
// Renders a row of die boxes, one per die in the pool. Each box shows:
//   top row:    die type label (D6, W6, C6, D8)
//   middle row: current face value (or "W" for wild)
//   bottom row: "HELD" if held, blank otherwise
//
// Example (5 dice, second and fourth held):
//   +--+ +--+ +--+ +--+ +--+
//   |D6| |D6| |W6| |D6| |D6|
//   | 5| | 3| | W| | 6| | 1|
//   |  | |HE| |  | |HE| |  |
//   +--+ +--+ +--+ +--+ +--+

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::Widget,
};

use crate::dice::DicePool;

pub struct DiceView<'a> {
    pub pool: &'a DicePool,
}

impl<'a> DiceView<'a> {
    pub fn new(pool: &'a DicePool) -> Self {
        Self { pool }
    }
}

impl Widget for DiceView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // for each die in self.pool.dice:
        //   draw a bordered box at the appropriate x offset
        //   render die.label() in the top row
        //   render die.display_value() centred in the middle row
        //   render "HE" (held indicator) in the bottom row if die.held
        todo!()
    }
}
