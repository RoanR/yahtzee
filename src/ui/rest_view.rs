// Rest site option list widget.
//
// Displays the three campfire options with a cursor, styled like ShopView.
//
//   Heal        Restore 15 HP                    <  (cursor)
//   Augment     Upgrade a face (+1 value)
//   Enchant     Upgrade a face (+5 score)
//
// Each option has a rounded box drawn around it.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    symbols,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

const ITEM_H: u16 = 4;

const OPTIONS: [(&str, &str); 3] = [
    ("Heal", "Restore 15 HP"),
    ("Augment", "Upgrade a face (+1 value)"),
    ("Enchant", "Upgrade a face (+5 score)"),
];

pub struct RestView {
    cursor: usize,
}

impl RestView {
    pub fn new(cursor: usize) -> Self {
        Self { cursor }
    }
}

impl Widget for RestView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for (i, (name, desc)) in OPTIONS.iter().enumerate() {
            let y = area.y + i as u16 * ITEM_H;
            if y + ITEM_H > area.bottom() {
                break;
            }

            let mut block = Block::bordered().border_set(symbols::border::ROUNDED);
            if i == self.cursor {
                block = block.style(Style::new().cyan());
            }

            let text: Vec<Line> = vec![
                Line::from(format!("  {name}")),
                Line::from(format!("  {desc}")),
            ];
            Paragraph::new(text)
                .block(block)
                .render(Rect::new(area.x + 1, y, area.right() - 1, ITEM_H), buf);
        }
    }
}
