// Main menu screen: the app's entry point, shown before a run starts.
//
// Hook point for future character-select and meta-progression-upgrade
// screens: those would add options here rather than replacing this phase.

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Paragraph,
};

use crate::game::GamePhase;

use super::{App, Phase, is_quit, wrap_cursor};

// Each option carries its own action rather than being keyed by index in
// handle_key: a future option inserted between New Run and Quit (per the
// module comment above) then gets whatever action it's given here, instead
// of silently falling into a catch-all that quits the app.
#[derive(Clone, Copy)]
enum MenuAction {
    NewRun,
    Quit,
}

const OPTIONS: [(&str, MenuAction); 2] =
    [("New Run", MenuAction::NewRun), ("Quit", MenuAction::Quit)];

pub(super) struct MainMenuPhase {
    pub cursor: usize,
}

impl Phase for MainMenuPhase {
    fn render(&self, _app: &App, frame: &mut ratatui::Frame) {
        let [title_area, menu_area, hints_area] = Layout::vertical([
            Constraint::Length(6),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .areas(frame.area());

        frame.render_widget(Paragraph::new("DUNGEON DICE").centered(), title_area);

        let lines: Vec<String> = OPTIONS
            .iter()
            .enumerate()
            .map(|(i, (label, _))| {
                if i == self.cursor {
                    format!("> {label}")
                } else {
                    format!("  {label}")
                }
            })
            .collect();
        frame.render_widget(Paragraph::new(lines.join("\n")).centered(), menu_area);

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
            KeyCode::Enter => match OPTIONS[self.cursor].1 {
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
