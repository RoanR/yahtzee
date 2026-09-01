// Rest site option list widget.
//
// Displays the three campfire options with a cursor, styled like ShopView.
//
//   Heal        Restore 15 HP                    <  (cursor)
//   Augment     Upgrade a face (+1 value)
//   Enchant     Upgrade a face (+5 score)
//
// Each option has a rounded box drawn around it.

use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    symbols,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::game::{GamePhase, UpgradeKind};

use super::{App, Phase};

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

pub(super) struct RestPhase {
    pub cursor: usize,
}

impl Phase for RestPhase {
    fn render(&self, app: &App, frame: &mut ratatui::Frame) {
        app.render_rest_shop(frame, RestView::new(self.cursor));
    }

    fn handle_key(&self, app: &mut App, code: KeyCode) -> bool {
        app.handle_rest_shop(code, self.cursor, 3, choose)
    }
}

fn choose(app: &mut App, cursor: usize) {
    match cursor {
        0 => {
            app.state.heal(15);
            app.state.dungeon.current_floor_mut().advance();
            app.transition_after_advance();
        }
        1 => {
            app.state.phase = GamePhase::UpgradeSelectDie {
                die_cursor: 0,
                kind: UpgradeKind::Augment,
                from_shop: false,
                pending_die: None,
            };
        }
        _ => {
            app.state.phase = GamePhase::UpgradeSelectDie {
                die_cursor: 0,
                kind: UpgradeKind::Enchant,
                from_shop: false,
                pending_die: None,
            };
        }
    }
}
