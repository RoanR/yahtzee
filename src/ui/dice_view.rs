// Dice display widget.
//
// Layout (6 rows total):
//
//    D6   D6   W6   D6   D6     <- die type labels (above boxes)
//   +--+ +--+ +--+ +--+ +--+
//   | 5| | 3| | W| | 6| | 1|   <- die boxes, 4 wide x 3 tall (value only)
//   +--+ +--+ +--+ +--+ +--+
//   EMPTY ROW      EMPTY ROW
//
//   Rolls: [##-]   2 / 3
//
// Rolls indicator format:
//   [##-]   2 / 3
//   ^ bar of max_rolls chars: "#" per used roll, "-" per remaining roll
//   The bar always shows max_rolls characters.
//   Example: max=3, remaining=2 -> "[#--]   2 / 3"
//   Example: max=3, remaining=0 -> "[###]   0 / 3"

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    symbols,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::dice::DicePool;

// Each box is 5 wide (border + 5 content + border), 3 tall (border + value + border).
const DIE_W: u16 = 5;
const DIE_H: u16 = 3;
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
        let labels_y = area.y;
        let boxes_y = area.y + 1;
        let held_y = boxes_y + DIE_H;
        let rolls_y = held_y + 1;

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

            // Pass 1: die type label above the box.
            if labels_y < area.bottom() {
                Paragraph::new(die.label())
                    .centered()
                    .render(Rect::new(x, labels_y, DIE_W, 1), buf);
            }

            // Pass 2: die box with value only.
            let value_str = match self
                .animated_values
                .and_then(|v| v.get(i))
                .copied()
                .flatten()
            {
                Some(v) => v.to_string(),
                None => die.display_value(),
            };
            let mut block = if die.held {
                Block::bordered().border_set(symbols::border::DOUBLE)
            } else {
                Block::bordered().border_set(symbols::border::PLAIN)
            };

            if die.selected {
                block = block.style(Style::new().cyan());
            }
            let box_height = DIE_H.min(area.bottom().saturating_sub(boxes_y));
            if box_height > 0 {
                Paragraph::new(Line::from(format!("{:>2}", value_str)))
                    .block(block)
                    .render(Rect::new(x, boxes_y, DIE_W, box_height), buf);
            }
            Paragraph::new(Line::from("\n")).render(area, buf);
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
