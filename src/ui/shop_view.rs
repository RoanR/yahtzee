// Shop item list widget.
//
// Displays available shop items with cursor, price, and affordability styling.
//
//   Item Name      Description                         75g  <  (cursor, affordable)
//   HP Potion      Restore 15 HP                       40g     (affordable, no cursor)
//   Dragon Bones   D8 die: faces 1-8...                50g  x  (cannot afford, grayed)

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Paragraph, Widget},
};

use crate::shop::ShopItem;

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
                let is_cursor = i == self.cursor;
                let price = item.price;
                let can_afford = self.gold >= price;

                let marker = if is_cursor {
                    "<"
                } else if !can_afford {
                    "x"
                } else {
                    " "
                };

                let style = if is_cursor {
                    Style::new().fg(Color::Cyan).bold()
                } else if !can_afford {
                    Style::new().fg(Color::DarkGray)
                } else {
                    Style::new()
                };

                let name = item.name();
                let desc = item.description();
                let text = format!("  {:<14} {:<40} {:>4}g  {}", name, desc, price, marker);
                lines.push(Line::styled(text, style));
            }
        }

        Paragraph::new(lines).render(area, buf);
    }
}
