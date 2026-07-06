// Dice display widget.
//
// Option A layout: die boxes in the top portion of the area, rolls indicator
// in the remaining rows below.
//
//   +--+ +--+ +--+ +--+ +--+
//   |D6| |D6| |W6| |D6| |D6|
//   | 5| | 3| | W| | 6| | 1|
//   |  | |HE| |  | |HE| |  |
//   +--+ +--+ +--+ +--+ +--+
//
//   Rolls: [##-]   2 / 3
//
// The caller must give this widget an area at least DIE_H + 2 rows tall so the
// rolls indicator fits. render_game allocates Min(0) for the left panel; the
// indicator uses the first row below the die boxes.
//
// Rolls indicator format:
//   [##-]   2 / 3
//   ^ bar of max_rolls chars: "#" per remaining roll, "-" per used roll
//   The bar always shows max_rolls characters.
//   Example: max=3, remaining=2 -> "[##-]   2 / 3"
//   Example: max=3, remaining=0 -> "[---]   0 / 3"

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
    pool: &'a DicePool,
    // Per-die animated display value. None = show actual die value (held or no animation).
    animated_values: Option<&'a [Option<u8>]>,
}

impl<'a> DiceView<'a> {
    pub fn new(pool: &'a DicePool) -> Self {
        Self {
            pool,
            animated_values: None,
        }
    }

    pub fn animated(pool: &'a DicePool, values: &'a [Option<u8>]) -> Self {
        Self {
            pool,
            animated_values: Some(values),
        }
    }
}

impl Widget for DiceView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let dice_height = DIE_H.min(area.height);
        let rolls_y = area.y + dice_height;
        let rolls_area = if rolls_y < area.bottom() {
            Some(Rect::new(area.x, rolls_y, area.width, 1))
        } else {
            None
        };

        for (i, die) in self.pool.dice.iter().enumerate() {
            let x = area.x + i as u16 * (DIE_W + GAP);
            if x + DIE_W > area.right() {
                break;
            }
            let die_area = Rect::new(x, area.y, DIE_W, dice_height);

            let value_str = match self
                .animated_values
                .and_then(|v| v.get(i))
                .copied()
                .flatten()
            {
                Some(v) => v.to_string(),
                None => die.display_value(),
            };

            let lines = vec![
                Line::from(die.label()),
                // Right-align value in the 2-char inner width: "| 5|", "| W|"
                Line::from(format!("{:>2}", value_str)),
                Line::from(if die.held { "HE" } else { "  " }),
            ];

            let block = if die.held {
                Block::bordered().bold()
            } else {
                Block::bordered()
            };

            Paragraph::new(lines).block(block).render(die_area, buf);
        }

        if let Some(rolls_area) = rolls_area {
            let remaining = self.pool.rolls_remaining as usize;
            let max = self.pool.max_rolls as usize;
            let used = max.saturating_sub(remaining);
            // Used pips first so the bar depletes left-to-right as rolls are spent.
            let bar = format!("[{}{}]", "#".repeat(used), "-".repeat(remaining));
            let label = format!("Rolls: {}   {} / {}", bar, remaining, max);
            Paragraph::new(label).render(rolls_area, buf);
        }
    }
}
