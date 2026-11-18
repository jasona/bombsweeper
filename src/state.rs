use rune::Tx;
use rune::prefs::Prefs;
use serde::{Deserialize, Serialize};

use crate::art::MascotMood;
use crate::game::{Board, CellState, Difficulty, GamePhase, Hint, RevealOutcome};
use crate::theme::ThemeKind;

const PREFS_APP_ID: &str = "dev.bombsweeper.app";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PressTarget {
    Tile(usize, usize),
    Chord(usize, usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modal {
    None,
    Stats,
    Custom,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Stats {
    pub best_beginner_ms: Option<u32>,
    pub best_intermediate_ms: Option<u32>,
    pub best_expert_ms: Option<u32>,
    pub played: u32,
    pub won: u32,
    pub streak: u32,
    pub best_streak: u32,
}

impl Stats {
    pub fn best_for(&self, diff: Difficulty) -> Option<u32> {
        match diff {
            Difficulty::Beginner => self.best_beginner_ms,
            Difficulty::Intermediate => self.best_intermediate_ms,
            Difficulty::Expert => self.best_expert_ms,
            Difficulty::Custom { .. } => None,
        }
    }

    pub fn record_win(&mut self, diff: Difficulty, elapsed_ms: u32, clean_run: bool) -> bool {
        self.played = self.played.saturating_add(1);
        self.won = self.won.saturating_add(1);
        self.streak = self.streak.saturating_add(1);
        self.best_streak = self.best_streak.max(self.streak);

        if !clean_run {
            return false;
        }
        let slot = match diff {
            Difficulty::Beginner => &mut self.best_beginner_ms,
            Difficulty::Intermediate => &mut self.best_intermediate_ms,
            Difficulty::Expert => &mut self.best_expert_ms,
            Difficulty::Custom { .. } => return false,
        };
        if slot.is_none_or(|prev| elapsed_ms < prev) {
            *slot = Some(elapsed_ms);
            true
        } else {
            false
        }
    }

    pub fn record_loss(&mut self) {
        self.played = self.played.saturating_add(1);
        self.streak = 0;
    }

    pub fn win_rate_pct(&self) -> u32 {
        if self.played == 0 {
            0
        } else {
            ((self.won as f64 / self.played as f64) * 100.0).round() as u32
        }
    }
}

pub fn format_ms(ms: u32) -> String {
    let secs = ms as f64 / 1000.0;
    if secs < 100.0 {
        format!("{secs:.2}s")
    } else {
        format!("{secs:.1}s")
    }
}

/// Ideal window content size `(width, height)` in logical points for any board difficulty.
pub fn window_size_for(diff: Difficulty) -> (f32, f32) {
    let (cols, rows, _) = diff.dims();
    let tile = diff.tile_size();
    let gap = diff.tile_gap();
    let board_w = cols as f32 * tile + cols.saturating_sub(1) as f32 * gap;
    let board_h = rows as f32 * tile + rows.saturating_sub(1) as f32 * gap;

    // Tray padding (20) + chassis padding (32) + window margin (44) = 96 horizontal.
    let w = (board_w + 96.0).max(504.0);
    // Top bar (30) + diff bar (30) + HUD (62) + progress (4) + footer (38) + 5 gaps (55)
    // + tray pad (20) + chassis pad (32) + window margin (46) = 317 vertical.
    let h = (board_h + 318.0).max(560.0);
    (w.ceil(), h.ceil())
}

pub struct AppState {
    pub board: Board,
    pub game_id: u64,
    pub theme: ThemeKind,
    pub flag_mode: bool,
    pub allow_question: bool,
    pub pressed: Option<PressTarget>,
    pub cursor: Option<(usize, usize)>,
    pub hint: Option<Hint>,
    pub hints_used: u32,
    pub modal: Modal,
    pub custom_cols: usize,
    pub custom_rows: usize,
    pub custom_bombs: usize,

    // Clock & timing
    pub started_at: Option<f64>,
    pub elapsed_secs: u32,
    pub exact_secs: f64,
    pub next_tick_at: Option<f64>,
    pub new_record: bool,

    // Stats & persistence
    pub stats: Stats,
    prefs: Prefs,
    pub deterministic: bool,
}

impl AppState {
    pub fn new() -> Self {
        let headless = rune::is_headless();
        let prefs = if headless {
            Prefs::memory()
        } else {
            Prefs::open(PREFS_APP_ID)
        };

        let theme = prefs.get::<ThemeKind>("theme").unwrap_or(ThemeKind::Obsidian);
        let allow_question = prefs.get::<bool>("allow_question").unwrap_or(false);
        let stats = prefs.get::<Stats>("stats").unwrap_or_default();

        let difficulty = Difficulty::Beginner;
        let seed = headless.then_some(2026_0518);

        Self {
            board: Board::new(difficulty, seed),
            game_id: 1,
            theme,
            flag_mode: false,
            allow_question,
            pressed: None,
            cursor: None,
            hint: None,
            hints_used: 0,
            modal: Modal::None,
            custom_cols: 12,
            custom_rows: 12,
            custom_bombs: 22,
            started_at: None,
            elapsed_secs: 0,
            exact_secs: 0.0,
            next_tick_at: None,
            new_record: false,
            stats,
            prefs,
            deterministic: headless,
        }
    }

    pub fn mascot_mood(&self) -> MascotMood {
        match self.board.phase {
            GamePhase::Won => MascotMood::Cool,
            GamePhase::Lost => MascotMood::Dead,
            GamePhase::Ready | GamePhase::Playing => {
                if self.pressed.is_some() {
                    MascotMood::Tense
                } else {
                    MascotMood::Happy
                }
            }
        }
    }

    pub fn is_tile_chord_highlighted(&self, r: usize, c: usize) -> bool {
        let Some(PressTarget::Chord(pr, pc)) = self.pressed else {
            return false;
        };
        if r == pr && c == pc {
            return false;
        }
        r.abs_diff(pr) <= 1
            && c.abs_diff(pc) <= 1
            && matches!(
                self.board.cell(r, c).state,
                CellState::Hidden | CellState::Question
            )
    }

    pub fn start_new_game(&mut self, difficulty: Difficulty, tx: &mut Tx) {
        let prev_dims = self.board.difficulty.dims();
        let next_dims = difficulty.dims();
        self.game_id = self.game_id.wrapping_add(1);
        let seed = self.deterministic.then_some(2026_0518u64.wrapping_add(self.game_id * 97));
        self.board = Board::new(difficulty, seed);
        self.pressed = None;
        self.hint = None;
        self.hints_used = 0;
        self.started_at = None;
        self.elapsed_secs = 0;
        self.exact_secs = 0.0;
        self.next_tick_at = None;
        self.new_record = false;
        self.modal = Modal::None;
        if let Some((cr, cc)) = self.cursor {
            self.cursor = Some((
                cr.min(self.board.rows - 1),
                cc.min(self.board.cols - 1),
            ));
        }

        if prev_dims.0 != next_dims.0 || prev_dims.1 != next_dims.1 {
            tx.resize_window(window_size_for(difficulty));
        }
    }

    pub fn restart(&mut self, tx: &mut Tx) {
        let diff = self.board.difficulty;
        self.start_new_game(diff, tx);
    }

    pub fn set_theme(&mut self, theme: ThemeKind, tx: &mut Tx) {
        if self.theme == theme {
            return;
        }
        self.theme = theme;
        tx.color_motion(rune::tok::motion::CALM);
        let _ = self.prefs.set("theme", &self.theme);
    }

    pub fn toggle_question_marks(&mut self) {
        self.allow_question = !self.allow_question;
        let _ = self.prefs.set("allow_question", &self.allow_question);
    }

    pub fn handle_primary_tile(&mut self, r: usize, c: usize, tx: &mut Tx) {
        if self.modal != Modal::None {
            return;
        }
        let cell_state = self.board.cell(r, c).state;
        if cell_state == CellState::Revealed {
            self.perform_chord(r, c, tx);
            return;
        }
        if self.flag_mode || tx.mods().shift {
            self.perform_flag(r, c);
        } else {
            self.perform_reveal(r, c, tx);
        }
    }

    pub fn handle_secondary_tile(&mut self, r: usize, c: usize, tx: &mut Tx) {
        if self.modal != Modal::None {
            return;
        }
        let cell_state = self.board.cell(r, c).state;
        if cell_state == CellState::Revealed {
            self.perform_chord(r, c, tx);
            return;
        }
        if self.flag_mode {
            self.perform_reveal(r, c, tx);
        } else {
            self.perform_flag(r, c);
        }
    }

    pub fn perform_flag(&mut self, r: usize, c: usize) {
        if self.board.toggle_flag(r, c, self.allow_question) {
            self.hint = None;
        }
    }

    pub fn perform_reveal(&mut self, r: usize, c: usize, tx: &mut Tx) {
        let was_ready = self.board.phase == GamePhase::Ready;
        let outcome = self.board.reveal(r, c);
        self.after_reveal_outcome(was_ready, outcome, tx);
    }

    pub fn perform_chord(&mut self, r: usize, c: usize, tx: &mut Tx) {
        let outcome = self.board.chord(r, c);
        self.after_reveal_outcome(false, outcome, tx);
    }

    fn after_reveal_outcome(&mut self, was_ready: bool, outcome: RevealOutcome, tx: &mut Tx) {
        if outcome == RevealOutcome::NoOp {
            return;
        }
        self.hint = None;
        let now = tx.clock();

        if was_ready && self.started_at.is_none() {
            self.started_at = Some(now);
            self.elapsed_secs = 0;
            self.exact_secs = 0.0;
        }

        match outcome {
            RevealOutcome::Opened { .. } => {
                if self.board.phase == GamePhase::Playing {
                    let start = self.started_at.unwrap_or(now);
                    let next_whole = ((now - start).floor() + 1.0).max(1.0);
                    let due = start + next_whole;
                    self.next_tick_at = Some(due);
                    tx.wake_at(due);
                }
            }
            RevealOutcome::Won { .. } => {
                let start = self.started_at.unwrap_or(now);
                self.exact_secs = (now - start).max(0.01);
                self.elapsed_secs = (self.exact_secs.floor() as u32).min(999);
                self.next_tick_at = None;
                let elapsed_ms = (self.exact_secs * 1000.0).round() as u32;
                self.new_record =
                    self.stats
                        .record_win(self.board.difficulty, elapsed_ms.max(10), self.hints_used == 0);
                let _ = self.prefs.set("stats", &self.stats);
                tx.announce(
                    format!("Victory! Cleared in {:.1} seconds.", self.exact_secs),
                    rune::LiveRegion::Polite,
                );
            }
            RevealOutcome::Exploded => {
                let start = self.started_at.unwrap_or(now);
                self.exact_secs = (now - start).max(0.0);
                self.elapsed_secs = (self.exact_secs.floor() as u32).min(999);
                self.next_tick_at = None;
                self.new_record = false;
                self.stats.record_loss();
                let _ = self.prefs.set("stats", &self.stats);
                tx.announce("Boom! You hit a bomb.", rune::LiveRegion::Assertive);
            }
            RevealOutcome::NoOp => {}
        }
    }

    pub fn request_hint(&mut self) {
        if self.modal != Modal::None {
            return;
        }
        if let Some(hint) = self.board.compute_hint() {
            if self.hint.map(|h| (h.row, h.col)) != Some((hint.row, hint.col))
                && self.board.phase == GamePhase::Playing
            {
                self.hints_used = self.hints_used.saturating_add(1);
            }
            self.hint = Some(hint);
        }
    }

    pub fn poll_timer(&mut self, clock: f64, tx: &mut Tx) -> bool {
        if self.board.phase != GamePhase::Playing {
            self.next_tick_at = None;
            return false;
        }
        let Some(start) = self.started_at else {
            return false;
        };
        let elapsed = (clock - start).max(0.0);
        let secs = (elapsed.floor() as u32).min(999);
        let changed = secs != self.elapsed_secs;
        if changed {
            self.elapsed_secs = secs;
            self.exact_secs = elapsed;
        }
        if self.elapsed_secs < 999 {
            let next_due = start + (secs + 1) as f64;
            self.next_tick_at = Some(next_due);
            tx.wake_at(next_due);
        } else {
            self.next_tick_at = None;
        }
        changed
    }

    pub fn reset_stats(&mut self) {
        self.stats = Stats::default();
        let _ = self.prefs.set("stats", &self.stats);
    }

    pub fn adjust_custom(&mut self, cols_delta: isize, rows_delta: isize, bombs_delta: isize) {
        self.custom_cols = (self.custom_cols as isize + cols_delta).clamp(6, 30) as usize;
        self.custom_rows = (self.custom_rows as isize + rows_delta).clamp(6, 20) as usize;
        let max_bombs = (self.custom_cols * self.custom_rows).saturating_sub(9).max(1);
        self.custom_bombs = (self.custom_bombs as isize + bombs_delta).clamp(1, max_bombs as isize) as usize;
    }

    pub fn apply_custom(&mut self, tx: &mut Tx) {
        let diff = Difficulty::Custom {
            cols: self.custom_cols,
            rows: self.custom_rows,
            bombs: self.custom_bombs,
        };
        self.start_new_game(diff, tx);
    }

    pub fn move_cursor(&mut self, dr: isize, dc: isize) {
        let (r, c) = self.cursor.unwrap_or((self.board.rows / 2, self.board.cols / 2));
        let nr = (r as isize + dr).rem_euclid(self.board.rows as isize) as usize;
        let nc = (c as isize + dc).rem_euclid(self.board.cols as isize) as usize;
        self.cursor = Some((nr, nc));
    }

    pub fn bv_per_sec(&self) -> f64 {
        if self.exact_secs <= 0.001 || self.board.board_3bv == 0 {
            0.0
        } else {
            self.board.board_3bv as f64 / self.exact_secs
        }
    }

    pub fn efficiency_pct(&self) -> u32 {
        if self.board.clicks == 0 || self.board.board_3bv == 0 {
            100
        } else {
            ((self.board.board_3bv as f64 / self.board.clicks as f64) * 100.0)
                .round()
                .clamp(1.0, 999.0) as u32
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
