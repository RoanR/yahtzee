// Path choice screen: rendered when the player is selecting their next room.
//
// Layout (content area):
//   Row 0-1: title "CHOOSE YOUR PATH" centered
//   Rows 2..end: two room panels side by side

use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::{
    dungeon::room::Room,
    game::{GamePhase, GameState},
};

use super::{App, Phase, is_quit};

pub struct PathView<'a> {
    state: &'a GameState,
    cursor: usize,
}

impl<'a> PathView<'a> {
    pub fn new(state: &'a GameState, cursor: usize) -> Self {
        Self { state, cursor }
    }
}

impl Widget for PathView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let floor = self.state.dungeon.current_floor();
        let options = match floor.next_options() {
            Some(o) => o,
            None => return,
        };

        let [title_area, panels_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(area);

        Paragraph::new("CHOOSE YOUR PATH")
            .centered()
            .render(title_area, buf);

        let [left_area, _gap, right_area] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(4),
            Constraint::Fill(1),
        ])
        .areas(panels_area);

        render_room_panel(&options[0], self.cursor == 0, left_area, buf);
        render_room_panel(&options[1], self.cursor == 1, right_area, buf);
    }
}

fn render_room_panel(room: &Room, selected: bool, area: Rect, buf: &mut Buffer) {
    let border_style = if selected {
        Style::new().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let (title, lines): (&str, Vec<Line>) = match room {
        Room::Challenge(t) => (
            "CHALLENGE",
            vec![
                Line::from(format!("Target: {} pts", t.required)),
                Line::from(format!("Reward: {}g", t.reward_gold)),
            ],
        ),
        Room::Elite(t) => (
            "ELITE",
            vec![
                Line::from(format!("Target: {} pts", t.required)),
                Line::from(format!("Reward: {}g", t.reward_gold)),
            ],
        ),
        Room::Rest => (
            "REST SITE",
            vec![Line::from("Heal 15 HP"), Line::from("Upgrade a die")],
        ),
    };

    let block = Block::new()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);

    let inner = block.inner(area);
    block.render(area, buf);
    Paragraph::new(lines).render(inner, buf);
}

pub(super) struct ChoosingRoomPhase {
    pub cursor: usize,
}

impl Phase for ChoosingRoomPhase {
    fn render(&self, app: &App, frame: &mut ratatui::Frame) {
        let content_area =
            app.vertical_layout(frame, "[Left/Right] Choose  [Enter] Confirm  [Q] Quit");
        frame.render_widget(PathView::new(&app.state, self.cursor), content_area);
    }

    fn handle_key(&self, app: &mut App, code: KeyCode) -> bool {
        if is_quit(code) {
            return false;
        }
        match code {
            KeyCode::Left | KeyCode::Up => {
                app.state.phase = GamePhase::ChoosingRoom { cursor: 0 };
                true
            }
            KeyCode::Right | KeyCode::Down => {
                app.state.phase = GamePhase::ChoosingRoom { cursor: 1 };
                true
            }
            KeyCode::Enter => {
                app.state.dungeon.current_floor_mut().choose(self.cursor);
                app.state.begin_room(true);
                app.state.phase = match app.state.dungeon.current_floor().current_room() {
                    Some(Room::Rest) => GamePhase::Rest { cursor: 0 },
                    _ => GamePhase::Rolling,
                };
                true
            }
            _ => true,
        }
    }
}
