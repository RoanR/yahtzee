// Dice display widget.
//
// Renders a row of die boxes, one per die in the pool. Each box shows:
//   top row:    die type label (D6, W6, C6, D8)
//   middle row: current face value (or "W" for wild), right-aligned
//   bottom row: "HE" if held, blank otherwise
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
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::dice::DicePool;

// Each box is 4 wide (border + 2 content + border), 5 tall (border + 3 rows + border).
const DIE_W: u16 = 4;
const DIE_H: u16 = 5;
const GAP: u16 = 1;

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
        for (i, die) in self.pool.dice.iter().enumerate() {
            let x = area.x + i as u16 * (DIE_W + GAP);
            if x + DIE_W > area.right() {
                break;
            }
            let die_area = Rect::new(x, area.y, DIE_W, DIE_H.min(area.height));

            let lines = vec![
                Line::from(die.label()),
                // Right-align value in the 2-char inner width: "| 5|", "| W|"
                Line::from(format!("{:>2}", die.display_value())),
                Line::from(if die.held { "HE" } else { "  " }),
            ];

            let block = if die.held {
                Block::bordered().bold()
            } else {
                Block::bordered()
            };

            Paragraph::new(lines).block(block).render(die_area, buf);
        }
    }
}
