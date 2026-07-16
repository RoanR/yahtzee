// Shop item list widget.
//
// Displays available shop items with cursor, price, and affordability styling.
//
//   Item Name      Description                         75g  <  (cursor, affordable)
//   HP Potion      Restore 15 HP                       40g     (affordable, no cursor)
//   Dragon Bones   D8 die: faces 1-8...                50g  x  (cannot afford, grayed)
//
// Each item has a rounded box drawn around it

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    symbols,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::shop::ShopItem;

// Each box is 4 tall (border + title/value + description + border)
// Each box is the width of the passed area -2
const ITEM_H: u16 = 4;
const GAP: u16 = 1;

pub struct ShopView<'a> {
    items: &'a [ShopItem],
    gold: u32,
    cursor: usize,
}

impl<'a> ShopView<'a> {
    pub fn new(items: &'a [ShopItem], gold: u32, cursor: usize) -> Self {
        Self {
            items,
            gold,
            cursor,
        }
    }
}

impl Widget for ShopView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut lines: Vec<Line> = vec![
            Line::from(""),
            Line::from(format!("  Gold: {}g", self.gold)).bold(),
            Line::from(""),
        ];

        if self.items.is_empty() {
            lines.push(Line::from("  Nothing for sale.").fg(Color::DarkGray));
        } else {
            for (i, item) in self.items.iter().enumerate() {
                let y = area.y + i as u16 * ITEM_H;
                if y + ITEM_H > area.bottom() {
                    break;
                }

                let mut block = Block::bordered().border_set(symbols::border::ROUNDED);
                let is_cursor = i == self.cursor;
                let price = format!("{}g", item.price);
                let can_afford = self.gold >= item.price;

                if is_cursor {
                    block = block.style(Style::new().cyan());
                } else if !can_afford {
                    block = block.style(Style::new().red());
                }

                let name = item.name();
                let desc = item.description();
                let text: Vec<Line> = vec![
                    Line::from(format!("  {:<14} {:>}", name, price)),
                    Line::from(format!("  {desc}  ")),
                ];
                Paragraph::new(text)
                    .block(block)
                    .render(Rect::new(area.x + 1, y, area.right() - 1, ITEM_H), buf);
            }
        }
    }
}
