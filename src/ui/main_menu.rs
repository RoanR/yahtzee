// Main menu screen: the app's entry point, shown before a run starts.
//
// Hook point for future character-select and meta-progression-upgrade
// screens: those would add options here rather than replacing this phase.

use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

use crate::game::GamePhase;

use super::{App, Phase, is_quit, wrap_cursor};

const BANNER: &str = r#"     ____              _                        ____  _
    |  _ \ _   _ _ __ | | ___  _ __   _ __     |  _ \(_) ___ ___
    | | | | | | | '_ \| |/ _ \| '_ \ | '_ \    | | | | |/ __/ _ \
    | |_| | |_| | | | | | (_) | | | || | | |   | |_| | | (_|  __/
    |____/ \__,_|_| |_|_|\___/|_| |_||_| |_|   |____/|_|\___\___|"#;

// Each option carries its own action rather than being keyed by index in
// handle_key: a future option inserted between New Run and Quit (per the
// module comment above) then gets whatever action it's given here, instead
// of silently falling into a catch-all that quits the app.
#[derive(Clone, Copy)]
enum MenuAction {
    NewRun,
    Quit,
}

const OPTIONS: [(&str, &str, MenuAction); 2] = [
    ("New Run", "Descend into the dungeon", MenuAction::NewRun),
    ("Quit", "Exit to desktop", MenuAction::Quit),
];

// Each card is 4 tall (border + title + description + border), matching
// ITEM_H in shop.rs/rest.rs.
const ITEM_H: u16 = 4;
const CARD_COL_WIDTH: u16 = 48;

pub(super) struct MainMenuPhase {
    pub cursor: usize,
}

impl Phase for MainMenuPhase {
    fn render(&self, _app: &App, frame: &mut ratatui::Frame) {
        let [content_area, hints_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());

        let banner_h = BANNER.lines().count() as u16;
        let banner_w = BANNER.lines().map(str::len).max().unwrap_or(0) as u16;
        let options_h = ITEM_H * OPTIONS.len() as u16;
        let block_h = banner_h + 1 + options_h;

        let [_, middle_area, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(block_h),
            Constraint::Fill(1),
        ])
        .areas(content_area);

        let [banner_area, _gap, options_col_area] = Layout::vertical([
            Constraint::Length(banner_h),
            Constraint::Length(1),
            Constraint::Length(options_h),
        ])
        .areas(middle_area);

        // Left-aligned within a box sized to the art's own longest line, then
        // that box is centered as a whole — centering each line independently
        // (as `.centered()` would) skews rows of differing width against
        // each other and distorts the art.
        let [_, banner_col_area, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(banner_w),
            Constraint::Fill(1),
        ])
        .areas(banner_area);
        frame.render_widget(Paragraph::new(BANNER), banner_col_area);

        let [_, options_area, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(CARD_COL_WIDTH),
            Constraint::Fill(1),
        ])
        .areas(options_col_area);

        frame.render_widget(OptionsView { cursor: self.cursor }, options_area);

        frame.render_widget(
            Paragraph::new("[Up/Down] Select  [Enter] Confirm  [Q] Quit"),
            hints_area,
        );
    }

    fn handle_key(&self, app: &mut App, code: KeyCode) -> bool {
        if is_quit(code) {
            return false;
        }
        let len = OPTIONS.len();
        match code {
            KeyCode::Up => {
                app.state
                    .phase
                    .set_cursor(wrap_cursor(self.cursor, len, false));
                true
            }
            KeyCode::Down => {
                app.state
                    .phase
                    .set_cursor(wrap_cursor(self.cursor, len, true));
                true
            }
            KeyCode::Enter => match OPTIONS[self.cursor].2 {
                MenuAction::NewRun => {
                    app.state.phase = GamePhase::ChoosingRoom { cursor: 0 };
                    true
                }
                MenuAction::Quit => false,
            },
            _ => true,
        }
    }
}

struct OptionsView {
    cursor: usize,
}

impl Widget for OptionsView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for (i, (name, desc, _)) in OPTIONS.iter().enumerate() {
            let y = area.y + i as u16 * ITEM_H;
            if y + ITEM_H > area.bottom() {
                break;
            }

            let selected = i == self.cursor;
            let border_style = if selected {
                Style::new().fg(Color::Cyan)
            } else {
                Style::default()
            };
            let title_style = if selected {
                Style::new().fg(Color::Cyan).bold()
            } else {
                Style::default()
            };

            let block = Block::bordered()
                .border_set(symbols::border::ROUNDED)
                .border_style(border_style)
                .title_top(Line::from(*name).style(title_style).centered());

            Paragraph::new(format!("  {desc}"))
                .block(block)
                .render(Rect::new(area.x, y, area.width, ITEM_H), buf);
        }
    }
}
