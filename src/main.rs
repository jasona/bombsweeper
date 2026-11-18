mod art;
mod game;
mod state;
mod theme;
mod view;

use rune::{AppSpec, Key, KeyEvent, Tx};

use crate::game::{Board, CellState, Difficulty, GamePhase};
use crate::state::{AppState, Modal, PressTarget, window_size_for};
use crate::theme::ThemeKind;

fn poll(s: &mut AppState, clock: f64, tx: &mut Tx) -> bool {
    s.poll_timer(clock, tx)
}

fn on_key(s: &mut AppState, ev: &KeyEvent, tx: &mut Tx) -> bool {
    if ev.mods.cmd || ev.mods.ctrl || ev.mods.alt {
        if ev.is_command('r') || ev.is_command('n') {
            s.restart(tx);
            return true;
        }
        return false;
    }

    if s.modal != Modal::None {
        if ev.key == Key::Escape {
            s.modal = Modal::None;
            return true;
        }
        return false;
    }

    match &ev.key {
        Key::Escape => {
            if s.hint.is_some() || s.cursor.is_some() {
                s.hint = None;
                s.cursor = None;
                true
            } else {
                false
            }
        }
        Key::Up => {
            s.move_cursor(-1, 0);
            true
        }
        Key::Down => {
            s.move_cursor(1, 0);
            true
        }
        Key::Left => {
            s.move_cursor(0, -1);
            true
        }
        Key::Right => {
            s.move_cursor(0, 1);
            true
        }
        Key::Enter => {
            if matches!(s.board.phase, GamePhase::Won | GamePhase::Lost) {
                s.restart(tx);
                true
            } else if let Some((r, c)) = s.cursor {
                s.handle_primary_tile(r, c, tx);
                true
            } else {
                false
            }
        }
        Key::Text(t) => match t.to_ascii_lowercase().as_str() {
            " " => {
                if matches!(s.board.phase, GamePhase::Won | GamePhase::Lost) {
                    s.restart(tx);
                } else if let Some((r, c)) = s.cursor {
                    s.handle_primary_tile(r, c, tx);
                } else {
                    s.cursor = Some((s.board.rows / 2, s.board.cols / 2));
                }
                true
            }
            "w" => {
                s.move_cursor(-1, 0);
                true
            }
            "s" => {
                s.move_cursor(1, 0);
                true
            }
            "a" => {
                s.move_cursor(0, -1);
                true
            }
            "d" => {
                s.move_cursor(0, 1);
                true
            }
            "f" => {
                if let Some((r, c)) = s.cursor {
                    s.perform_flag(r, c);
                    true
                } else {
                    false
                }
            }
            "h" => {
                s.request_hint();
                true
            }
            "m" => {
                s.flag_mode = !s.flag_mode;
                true
            }
            "r" | "n" => {
                s.restart(tx);
                true
            }
            "t" => {
                let next = s.theme.next();
                s.set_theme(next, tx);
                true
            }
            "1" => {
                s.start_new_game(Difficulty::Beginner, tx);
                true
            }
            "2" => {
                s.start_new_game(Difficulty::Intermediate, tx);
                true
            }
            "3" => {
                s.start_new_game(Difficulty::Expert, tx);
                true
            }
            _ => false,
        },
        _ => false,
    }
}

fn scene_midgame(s: &mut AppState) {
    s.theme = ThemeKind::Obsidian;
    s.board = Board::demo_midgame(Difficulty::Beginner, 42);
    s.elapsed_secs = 27;
    s.exact_secs = 27.4;
}

fn scene_alabaster(s: &mut AppState) {
    s.theme = ThemeKind::Alabaster;
    s.board = Board::demo_midgame(Difficulty::Beginner, 42);
    s.elapsed_secs = 27;
    s.exact_secs = 27.4;
}

fn scene_classic95(s: &mut AppState) {
    s.theme = ThemeKind::Classic95;
    s.board = Board::demo_midgame(Difficulty::Beginner, 42);
    s.elapsed_secs = 27;
    s.exact_secs = 27.4;
}

fn scene_won(s: &mut AppState) {
    s.theme = ThemeKind::Obsidian;
    s.board = Board::demo_won(Difficulty::Beginner, 42);
    s.elapsed_secs = 14;
    s.exact_secs = 14.28;
    s.new_record = true;
}

fn scene_lost(s: &mut AppState) {
    s.theme = ThemeKind::Obsidian;
    s.board = Board::demo_lost(Difficulty::Beginner, 42);
    s.elapsed_secs = 31;
    s.exact_secs = 31.6;
}

fn scene_hint(s: &mut AppState) {
    scene_midgame(s);
    s.request_hint();
}

fn scene_tense(s: &mut AppState) {
    scene_midgame(s);
    // Find a revealed numbered tile with unrevealed neighbors for chord preview.
    for r in 0..s.board.rows {
        for c in 0..s.board.cols {
            let cell = s.board.cell(r, c);
            if cell.state == CellState::Revealed
                && cell.adjacent > 0
                && s.board
                    .neighbors(r, c)
                    .any(|(nr, nc)| s.board.cell(nr, nc).state == CellState::Hidden)
            {
                s.pressed = Some(PressTarget::Chord(r, c));
                return;
            }
        }
    }
}

fn scene_intermediate(s: &mut AppState) {
    s.theme = ThemeKind::Obsidian;
    s.board = Board::demo_midgame(Difficulty::Intermediate, 77);
    s.elapsed_secs = 64;
    s.exact_secs = 64.2;
}

fn scene_expert(s: &mut AppState) {
    s.theme = ThemeKind::Obsidian;
    s.board = Board::demo_midgame(Difficulty::Expert, 108);
    s.elapsed_secs = 118;
    s.exact_secs = 118.7;
}

fn scene_stats(s: &mut AppState) {
    scene_midgame(s);
    s.stats.best_beginner_ms = Some(14_280);
    s.stats.best_intermediate_ms = Some(58_940);
    s.stats.best_expert_ms = Some(142_610);
    s.stats.played = 34;
    s.stats.won = 26;
    s.stats.streak = 5;
    s.stats.best_streak = 9;
    s.modal = Modal::Stats;
}

fn scene_custom(s: &mut AppState) {
    scene_midgame(s);
    s.modal = Modal::Custom;
}

fn app_spec() -> AppSpec<AppState> {
    let initial = window_size_for(Difficulty::Beginner);
    AppSpec::<AppState> {
        min_size: Some((480.0, 560.0)),
        families: ("Inter", "ui-monospace"),
        poll: Some(poll),
        on_key: Some(on_key),
        scenes: &[
            ("default", |_| {}),
            ("midgame", scene_midgame),
            ("alabaster", scene_alabaster),
            ("classic95", scene_classic95),
            ("won", scene_won),
            ("lost", scene_lost),
            ("hint", scene_hint),
            ("tense", scene_tense),
            ("intermediate", scene_intermediate),
            ("expert", scene_expert),
            ("stats", scene_stats),
            ("custom", scene_custom),
        ],
        ..AppSpec::new("Bombsweeper", initial, AppState::new, view::view)
    }
}

fn main() {
    rune::set_app_id("dev.bombsweeper.app");
    rune::run(app_spec());
}

#[cfg(test)]
mod tests {
    use super::*;
    use rune::{Env, Mods, Runtime};

    fn test_runtime() -> Runtime<AppState> {
        let mut state = AppState::new();
        state.deterministic = true;
        state.board = Board::new(Difficulty::Beginner, Some(2026));
        let size = window_size_for(Difficulty::Beginner);
        let mut rt = Runtime::new(state, view::view, size, Env::new(true, 2.0));
        rt.poll = Some(poll);
        rt.on_key = Some(on_key);
        rt.frame(0.0);
        rt
    }

    #[test]
    fn clicking_a_tile_opens_board_and_starts_timer() {
        let mut rt = test_runtime();
        let (x, y) = rt.find("cell-4-4").expect("center tile should exist");
        rt.pointer_down(x, y);
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.mascot_mood(), art::MascotMood::Tense);

        rt.pointer_up(x, y);
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.board.phase, GamePhase::Playing);
        assert!(rt.state.board.revealed_safe >= 9);
        assert_eq!(rt.state.mascot_mood(), art::MascotMood::Happy);

        // Advance virtual clock by 3.2 seconds and confirm timer increments.
        for _ in 0..200 {
            rt.frame(1.0 / 60.0);
        }
        assert!(rt.state.elapsed_secs >= 3, "elapsed: {}", rt.state.elapsed_secs);
    }

    #[test]
    fn right_click_flags_tile_and_updates_bomb_counter() {
        let mut rt = test_runtime();
        let (x, y) = rt.find("cell-0-0").unwrap();
        rt.secondary_down(x, y);
        rt.secondary_up(x, y);
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.board.cell(0, 0).state, CellState::Flagged);
        assert_eq!(rt.state.board.bombs_remaining(), 9);
    }

    #[test]
    fn theme_and_difficulty_controls_update_state() {
        let mut rt = test_runtime();
        let (tx, ty) = rt.find("theme-95").unwrap();
        rt.pointer_down(tx, ty);
        rt.pointer_up(tx, ty);
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.theme, ThemeKind::Classic95);

        let (dx, dy) = rt.find("diff-intermediate").unwrap();
        rt.pointer_down(dx, dy);
        rt.pointer_up(dx, dy);
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.board.difficulty, Difficulty::Intermediate);
        assert_eq!(rt.state.board.cols, 16);
        assert_eq!(rt.state.board.rows, 16);
    }

    #[test]
    fn keyboard_controls_navigate_flag_hint_and_restart() {
        let mut rt = test_runtime();
        rt.key(KeyEvent { key: Key::Text(" ".into()), mods: Mods::default() });
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.cursor, Some((4, 4)));

        rt.key(KeyEvent { key: Key::Enter, mods: Mods::default() });
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.board.phase, GamePhase::Playing);

        rt.key(KeyEvent { key: Key::Text("h".into()), mods: Mods::default() });
        rt.frame(1.0 / 60.0);
        assert!(rt.state.hint.is_some());

        rt.key(KeyEvent { key: Key::Text("r".into()), mods: Mods::default() });
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.board.phase, GamePhase::Ready);
        assert!(rt.state.hint.is_none());
    }
}
