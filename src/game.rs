use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Seeded SplitMix64 PRNG — fast, zero-dependency, deterministic for tests and scenes.
#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
}

static ENTROPY_COUNTER: AtomicU64 = AtomicU64::new(1);

impl Rng {
    pub fn from_seed(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn from_entropy() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
        let seq = ENTROPY_COUNTER.fetch_add(1, Ordering::Relaxed);
        let mixed = nanos ^ seq.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        Self { state: mixed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn next_usize(&mut self, bound: usize) -> usize {
        debug_assert!(bound > 0);
        (self.next_u64() % bound as u64) as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Expert,
    Custom { cols: usize, rows: usize, bombs: usize },
}

impl Difficulty {
    pub const PRESETS: [Difficulty; 3] = [
        Difficulty::Beginner,
        Difficulty::Intermediate,
        Difficulty::Expert,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Difficulty::Beginner => "Beginner",
            Difficulty::Intermediate => "Intermediate",
            Difficulty::Expert => "Expert",
            Difficulty::Custom { .. } => "Custom",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Difficulty::Beginner => "9×9",
            Difficulty::Intermediate => "16×16",
            Difficulty::Expert => "30×16",
            Difficulty::Custom { .. } => "Custom",
        }
    }

    pub fn dims(self) -> (usize, usize, usize) {
        match self {
            Difficulty::Beginner => (9, 9, 10),
            Difficulty::Intermediate => (16, 16, 40),
            Difficulty::Expert => (30, 16, 99),
            Difficulty::Custom { cols, rows, bombs } => {
                let c = cols.clamp(6, 30);
                let r = rows.clamp(6, 20);
                let max_b = (c * r).saturating_sub(9).max(1);
                let b = bombs.clamp(1, max_b);
                (c, r, b)
            }
        }
    }

    pub fn tile_size(self) -> f32 {
        let (cols, rows, _) = self.dims();
        if cols <= 10 && rows <= 10 {
            42.0
        } else if cols <= 18 && rows <= 16 {
            31.0
        } else if cols <= 24 {
            28.0
        } else {
            26.0
        }
    }

    pub fn tile_gap(self) -> f32 {
        let (cols, _, _) = self.dims();
        if cols <= 10 {
            4.0
        } else if cols <= 18 {
            3.0
        } else {
            2.5
        }
    }

    pub fn preset_index(self) -> Option<usize> {
        match self {
            Difficulty::Beginner => Some(0),
            Difficulty::Intermediate => Some(1),
            Difficulty::Expert => Some(2),
            Difficulty::Custom { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellState {
    Hidden,
    Flagged,
    Question,
    Revealed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub has_bomb: bool,
    pub adjacent: u8,
    pub state: CellState,
    /// BFS ripple depth from the click that revealed this cell (or distance from blast).
    pub reveal_wave: u16,
    /// True only on the bomb that ended the game.
    pub detonated: bool,
    /// True on game over when a flag was placed on a safe tile.
    pub misflagged: bool,
    /// True on victory for bombs that snapped flags into place automatically.
    pub auto_flagged: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            has_bomb: false,
            adjacent: 0,
            state: CellState::Hidden,
            reveal_wave: 0,
            detonated: false,
            misflagged: false,
            auto_flagged: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamePhase {
    Ready,
    Playing,
    Won,
    Lost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RevealOutcome {
    NoOp,
    Opened { count: usize },
    Won { count: usize },
    Exploded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HintKind {
    Safe,
    Bomb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hint {
    pub row: usize,
    pub col: usize,
    pub kind: HintKind,
    pub reason: &'static str,
}

#[derive(Clone, Debug)]
pub struct Board {
    pub difficulty: Difficulty,
    pub cols: usize,
    pub rows: usize,
    pub bomb_count: usize,
    pub cells: Vec<Cell>,
    pub phase: GamePhase,
    pub flagged_count: usize,
    pub revealed_safe: usize,
    pub clicks: u32,
    pub board_3bv: u32,
    pub last_blast: Option<(usize, usize)>,
    rng: Rng,
}

impl Board {
    pub fn new(difficulty: Difficulty, seed: Option<u64>) -> Self {
        let (cols, rows, bomb_count) = difficulty.dims();
        let rng = match seed {
            Some(s) => Rng::from_seed(s),
            None => Rng::from_entropy(),
        };
        Self {
            difficulty,
            cols,
            rows,
            bomb_count,
            cells: vec![Cell::default(); cols * rows],
            phase: GamePhase::Ready,
            flagged_count: 0,
            revealed_safe: 0,
            clicks: 0,
            board_3bv: 0,
            last_blast: None,
            rng,
        }
    }

    #[inline]
    pub fn idx(&self, r: usize, c: usize) -> usize {
        r * self.cols + c
    }

    #[inline]
    pub fn cell(&self, r: usize, c: usize) -> &Cell {
        &self.cells[self.idx(r, c)]
    }

    #[inline]
    pub fn cell_mut(&mut self, r: usize, c: usize) -> &mut Cell {
        let i = self.idx(r, c);
        &mut self.cells[i]
    }

    pub fn safe_total(&self) -> usize {
        (self.cols * self.rows).saturating_sub(self.bomb_count)
    }

    pub fn bombs_remaining(&self) -> i32 {
        self.bomb_count as i32 - self.flagged_count as i32
    }

    pub fn progress(&self) -> f32 {
        let total = self.safe_total();
        if total == 0 {
            return 1.0;
        }
        (self.revealed_safe as f32 / total as f32).clamp(0.0, 1.0)
    }

    pub fn neighbors(&self, r: usize, c: usize) -> impl Iterator<Item = (usize, usize)> + use<> {
        let r_start = r.saturating_sub(1);
        let r_end = (r + 1).min(self.rows - 1);
        let c_start = c.saturating_sub(1);
        let c_end = (c + 1).min(self.cols - 1);
        (r_start..=r_end).flat_map(move |nr| {
            (c_start..=c_end).filter_map(move |nc| (!(nr == r && nc == c)).then_some((nr, nc)))
        })
    }

    pub fn adjacent_flags(&self, r: usize, c: usize) -> usize {
        self.neighbors(r, c)
            .filter(|&(nr, nc)| self.cell(nr, nc).state == CellState::Flagged)
            .count()
    }

    pub fn can_chord(&self, r: usize, c: usize) -> bool {
        if self.phase != GamePhase::Playing {
            return false;
        }
        let cell = self.cell(r, c);
        if cell.state != CellState::Revealed || cell.adjacent == 0 {
            return false;
        }
        let flags = self.adjacent_flags(r, c);
        let has_unopened = self.neighbors(r, c).any(|(nr, nc)| {
            matches!(self.cell(nr, nc).state, CellState::Hidden | CellState::Question)
        });
        flags == usize::from(cell.adjacent) && has_unopened
    }

    /// Places bombs ensuring the first-clicked cell (and its 3×3 neighborhood when space allows)
    /// is completely bomb-free so every game opens cleanly.
    fn place_bombs_avoiding(&mut self, safe_r: usize, safe_c: usize) {
        let total = self.cols * self.rows;
        let protect_neighborhood = total.saturating_sub(self.bomb_count) >= 9;

        let mut candidates: Vec<usize> = Vec::with_capacity(total);
        for r in 0..self.rows {
            for c in 0..self.cols {
                let dr = r.abs_diff(safe_r);
                let dc = c.abs_diff(safe_c);
                let excluded = if protect_neighborhood {
                    dr <= 1 && dc <= 1
                } else {
                    dr == 0 && dc == 0
                };
                if !excluded {
                    candidates.push(self.idx(r, c));
                }
            }
        }

        // Fisher-Yates partial shuffle for uniform bomb placement.
        let count = self.bomb_count.min(candidates.len());
        for i in 0..count {
            let j = i + self.rng.next_usize(candidates.len() - i);
            candidates.swap(i, j);
            let idx = candidates[i];
            self.cells[idx].has_bomb = true;
        }

        self.recompute_adjacencies_and_3bv();
    }

    pub(crate) fn recompute_adjacencies_and_3bv(&mut self) {
        for r in 0..self.rows {
            for c in 0..self.cols {
                let count = self
                    .neighbors(r, c)
                    .filter(|&(nr, nc)| self.cell(nr, nc).has_bomb)
                    .count() as u8;
                self.cell_mut(r, c).adjacent = count;
            }
        }
        self.board_3bv = self.compute_3bv();
    }

    /// Exact Bechtel's Board Benchmark Value (minimum left clicks to solve without flags).
    fn compute_3bv(&self) -> u32 {
        let total = self.cols * self.rows;
        let mut marked = vec![false; total];
        let mut bv = 0u32;
        let mut queue = VecDeque::new();

        for r in 0..self.rows {
            for c in 0..self.cols {
                let i = self.idx(r, c);
                let cell = self.cells[i];
                if cell.has_bomb || cell.adjacent != 0 || marked[i] {
                    continue;
                }
                bv += 1;
                marked[i] = true;
                queue.push_back((r, c));
                while let Some((cr, cc)) = queue.pop_front() {
                    for (nr, nc) in self.neighbors(cr, cc) {
                        let ni = self.idx(nr, nc);
                        if marked[ni] || self.cells[ni].has_bomb {
                            continue;
                        }
                        marked[ni] = true;
                        if self.cells[ni].adjacent == 0 {
                            queue.push_back((nr, nc));
                        }
                    }
                }
            }
        }

        for r in 0..self.rows {
            for c in 0..self.cols {
                let i = self.idx(r, c);
                if !self.cells[i].has_bomb && !marked[i] {
                    bv += 1;
                }
            }
        }
        bv.max(1)
    }

    pub fn reveal(&mut self, r: usize, c: usize) -> RevealOutcome {
        if r >= self.rows || c >= self.cols {
            return RevealOutcome::NoOp;
        }
        if matches!(self.phase, GamePhase::Won | GamePhase::Lost) {
            return RevealOutcome::NoOp;
        }
        if matches!(self.cell(r, c).state, CellState::Flagged | CellState::Revealed) {
            return RevealOutcome::NoOp;
        }

        if self.phase == GamePhase::Ready {
            self.place_bombs_avoiding(r, c);
            self.phase = GamePhase::Playing;
        }

        self.clicks = self.clicks.saturating_add(1);
        self.reveal_internal(&[(r, c)], (r, c))
    }

    pub fn chord(&mut self, r: usize, c: usize) -> RevealOutcome {
        if r >= self.rows || c >= self.cols || !self.can_chord(r, c) {
            return RevealOutcome::NoOp;
        }
        let targets: Vec<(usize, usize)> = self
            .neighbors(r, c)
            .filter(|&(nr, nc)| {
                matches!(self.cell(nr, nc).state, CellState::Hidden | CellState::Question)
            })
            .collect();
        if targets.is_empty() {
            return RevealOutcome::NoOp;
        }
        self.clicks = self.clicks.saturating_add(1);
        self.reveal_internal(&targets, (r, c))
    }

    fn reveal_internal(&mut self, seeds: &[(usize, usize)], origin: (usize, usize)) -> RevealOutcome {
        let mut exploded_at: Option<(usize, usize)> = None;
        for &(r, c) in seeds {
            if self.cell(r, c).has_bomb {
                self.cell_mut(r, c).state = CellState::Revealed;
                self.cell_mut(r, c).detonated = true;
                self.cell_mut(r, c).reveal_wave = 0;
                if exploded_at.is_none() {
                    exploded_at = Some((r, c));
                }
            }
        }

        if let Some((br, bc)) = exploded_at {
            self.trigger_loss(br, bc);
            return RevealOutcome::Exploded;
        }

        let mut queue = VecDeque::new();
        let mut opened = 0usize;

        for &(r, c) in seeds {
            let i = self.idx(r, c);
            if matches!(self.cells[i].state, CellState::Hidden | CellState::Question) {
                self.cells[i].state = CellState::Revealed;
                self.cells[i].reveal_wave = 0;
                let adj = self.cells[i].adjacent;
                self.revealed_safe += 1;
                opened += 1;
                if adj == 0 {
                    queue.push_back((r, c, 0u16));
                }
            }
        }

        while let Some((cr, cc, wave)) = queue.pop_front() {
            for (nr, nc) in self.neighbors(cr, cc) {
                let ni = self.idx(nr, nc);
                let cell = &mut self.cells[ni];
                if cell.has_bomb || cell.state == CellState::Revealed || cell.state == CellState::Flagged {
                    continue;
                }
                cell.state = CellState::Revealed;
                cell.reveal_wave = wave.saturating_add(1);
                let adj = cell.adjacent;
                self.revealed_safe += 1;
                opened += 1;
                if adj == 0 {
                    queue.push_back((nr, nc, wave.saturating_add(1)));
                }
            }
        }

        if self.revealed_safe >= self.safe_total() {
            self.trigger_win(origin);
            return RevealOutcome::Won { count: opened };
        }

        if opened > 0 {
            RevealOutcome::Opened { count: opened }
        } else {
            RevealOutcome::NoOp
        }
    }

    fn trigger_loss(&mut self, blast_r: usize, blast_c: usize) {
        self.phase = GamePhase::Lost;
        self.last_blast = Some((blast_r, blast_c));
        for r in 0..self.rows {
            for c in 0..self.cols {
                let dist = (r.abs_diff(blast_r) + c.abs_diff(blast_c)) as u16;
                let cell = self.cell_mut(r, c);
                if cell.has_bomb && cell.state != CellState::Flagged && !cell.detonated {
                    cell.state = CellState::Revealed;
                    cell.reveal_wave = dist.max(1);
                } else if !cell.has_bomb && cell.state == CellState::Flagged {
                    cell.misflagged = true;
                }
            }
        }
    }

    fn trigger_win(&mut self, origin: (usize, usize)) {
        self.phase = GamePhase::Won;
        for r in 0..self.rows {
            for c in 0..self.cols {
                let dist = (r.abs_diff(origin.0) + c.abs_diff(origin.1)) as u16;
                let cell = self.cell_mut(r, c);
                if cell.has_bomb && cell.state != CellState::Flagged {
                    cell.state = CellState::Flagged;
                    cell.auto_flagged = true;
                    cell.reveal_wave = dist;
                    self.flagged_count += 1;
                }
            }
        }
    }

    pub fn toggle_flag(&mut self, r: usize, c: usize, allow_question: bool) -> bool {
        if r >= self.rows || c >= self.cols {
            return false;
        }
        if matches!(self.phase, GamePhase::Won | GamePhase::Lost) {
            return false;
        }
        let i = self.idx(r, c);
        match self.cells[i].state {
            CellState::Hidden => {
                self.cells[i].state = CellState::Flagged;
                self.flagged_count += 1;
                true
            }
            CellState::Flagged => {
                self.flagged_count = self.flagged_count.saturating_sub(1);
                self.cells[i].state = if allow_question {
                    CellState::Question
                } else {
                    CellState::Hidden
                };
                true
            }
            CellState::Question => {
                self.cells[i].state = CellState::Hidden;
                true
            }
            CellState::Revealed => false,
        }
    }

    /// Computes a smart logical hint using single-cell and pairwise subset deduction,
    /// falling back to a guaranteed safe cell if the current position requires a guess.
    pub fn compute_hint(&self) -> Option<Hint> {
        if matches!(self.phase, GamePhase::Won | GamePhase::Lost) {
            return None;
        }
        if self.phase == GamePhase::Ready {
            return Some(Hint {
                row: self.rows / 2,
                col: self.cols / 2,
                kind: HintKind::Safe,
                reason: "Start in the center — first click is always safe",
            });
        }

        // Gather frontier constraints: (row, col, remaining_bombs, unknown_neighbors).
        type Constraint = (usize, usize, i32, Vec<(usize, usize)>);
        let mut constraints: Vec<Constraint> = Vec::new();
        for r in 0..self.rows {
            for c in 0..self.cols {
                let cell = self.cell(r, c);
                if cell.state != CellState::Revealed || cell.adjacent == 0 {
                    continue;
                }
                let mut flags = 0i32;
                let mut unknown = Vec::new();
                for (nr, nc) in self.neighbors(r, c) {
                    match self.cell(nr, nc).state {
                        CellState::Flagged => flags += 1,
                        CellState::Hidden | CellState::Question => unknown.push((nr, nc)),
                        CellState::Revealed => {}
                    }
                }
                if unknown.is_empty() {
                    continue;
                }
                let rem = i32::from(cell.adjacent) - flags;
                if rem == 0 {
                    let (hr, hc) = unknown[0];
                    return Some(Hint {
                        row: hr,
                        col: hc,
                        kind: HintKind::Safe,
                        reason: "All adjacent bombs are flagged — safe to open",
                    });
                }
                if rem == unknown.len() as i32 {
                    let (hr, hc) = unknown[0];
                    return Some(Hint {
                        row: hr,
                        col: hc,
                        kind: HintKind::Bomb,
                        reason: "Remaining neighbors must all be bombs — flag it",
                    });
                }
                if rem > 0 {
                    constraints.push((r, c, rem, unknown));
                }
            }
        }

        // Pairwise subset deduction (handles 1-2-1, 1-2-2-1, and overlap patterns).
        for i in 0..constraints.len() {
            let (r1, c1, rem1, ref u1) = constraints[i];
            for (j, &(r2, c2, rem2, ref u2)) in constraints.iter().enumerate() {
                if i == j || r1.abs_diff(r2) > 2 || c1.abs_diff(c2) > 2 {
                    continue;
                }
                if u1.len() < u2.len() && u1.iter().all(|p| u2.contains(p)) {
                    let diff: Vec<(usize, usize)> =
                        u2.iter().copied().filter(|p| !u1.contains(p)).collect();
                    let rem_diff = rem2 - rem1;
                    if rem_diff == 0 && !diff.is_empty() {
                        return Some(Hint {
                            row: diff[0].0,
                            col: diff[0].1,
                            kind: HintKind::Safe,
                            reason: "Deduced safe from overlapping neighbor clues",
                        });
                    }
                    if rem_diff == diff.len() as i32 && !diff.is_empty() {
                        return Some(Hint {
                            row: diff[0].0,
                            col: diff[0].1,
                            kind: HintKind::Bomb,
                            reason: "Deduced bomb from overlapping neighbor clues",
                        });
                    }
                }
            }
        }

        // Check if any flag on the board is actually wrong — alert subtly by pointing to a safe move,
        // preferring an unrevealed safe cell with adjacent == 0 or on the frontier.
        let mut best_safe: Option<(usize, usize, u8)> = None;
        for r in 0..self.rows {
            for c in 0..self.cols {
                let cell = self.cell(r, c);
                if cell.has_bomb || cell.state == CellState::Revealed || cell.state == CellState::Flagged {
                    continue;
                }
                let on_frontier = self
                    .neighbors(r, c)
                    .any(|(nr, nc)| self.cell(nr, nc).state == CellState::Revealed);
                let score = if cell.adjacent == 0 {
                    0
                } else if on_frontier {
                    1
                } else {
                    2
                };
                if best_safe.is_none_or(|(_, _, s)| score < s) {
                    best_safe = Some((r, c, score));
                }
            }
        }

        best_safe.map(|(row, col, _)| Hint {
            row,
            col,
            kind: HintKind::Safe,
            reason: "No forced deduction — here is a safe tile",
        })
    }

    /// Builds a deterministic mid-game board for visual snapshots and demos.
    pub fn demo_midgame(difficulty: Difficulty, seed: u64) -> Self {
        let mut board = Self::new(difficulty, Some(seed));
        let (sr, sc) = (board.rows / 2, board.cols / 2);
        board.reveal(sr, sc);
        // Apply a few deterministic logical moves so numbers, flags, and openings look lively.
        for _ in 0..10 {
            let Some(hint) = board.compute_hint() else { break };
            match hint.kind {
                HintKind::Bomb => {
                    board.toggle_flag(hint.row, hint.col, false);
                }
                HintKind::Safe => {
                    if board.revealed_safe * 5 >= board.safe_total() * 3 {
                        break;
                    }
                    board.reveal(hint.row, hint.col);
                }
            }
        }
        board
    }

    /// Builds a deterministic won board for visual snapshots.
    pub fn demo_won(difficulty: Difficulty, seed: u64) -> Self {
        let mut board = Self::new(difficulty, Some(seed));
        board.reveal(board.rows / 2, board.cols / 2);
        for r in 0..board.rows {
            for c in 0..board.cols {
                if !board.cell(r, c).has_bomb && board.cell(r, c).state != CellState::Revealed {
                    board.reveal(r, c);
                }
            }
        }
        board
    }

    /// Builds a deterministic lost board (with detonated mine, revealed mines, and a misflag).
    pub fn demo_lost(difficulty: Difficulty, seed: u64) -> Self {
        let mut board = Self::demo_midgame(difficulty, seed);
        // Place one deliberate misflag on a hidden safe cell so we showcase misflag art too.
        'outer_flag: for r in 0..board.rows {
            for c in 0..board.cols {
                let cell = board.cell(r, c);
                if !cell.has_bomb && cell.state == CellState::Hidden {
                    board.toggle_flag(r, c, false);
                    break 'outer_flag;
                }
            }
        }
        // Trigger an unflagged bomb near the middle.
        let mut target = None;
        let (mr, mc) = (board.rows / 2, board.cols / 2);
        let mut best_d = usize::MAX;
        for r in 0..board.rows {
            for c in 0..board.cols {
                let cell = board.cell(r, c);
                if cell.has_bomb && cell.state == CellState::Hidden {
                    let d = r.abs_diff(mr) + c.abs_diff(mc);
                    if d < best_d {
                        best_d = d;
                        target = Some((r, c));
                    }
                }
            }
        }
        if let Some((r, c)) = target {
            board.reveal(r, c);
        }
        board
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_click_and_neighbors_are_always_safe() {
        for seed in 1..40u64 {
            let mut board = Board::new(Difficulty::Beginner, Some(seed));
            let outcome = board.reveal(4, 4);
            assert!(matches!(outcome, RevealOutcome::Opened { .. } | RevealOutcome::Won { .. }));
            assert_eq!(board.phase, GamePhase::Playing);
            assert!(!board.cell(4, 4).has_bomb);
            assert_eq!(board.cell(4, 4).adjacent, 0);
            for (nr, nc) in board.neighbors(4, 4) {
                assert!(!board.cell(nr, nc).has_bomb);
            }
            let total_bombs = board.cells.iter().filter(|c| c.has_bomb).count();
            assert_eq!(total_bombs, 10);
            assert!(board.board_3bv >= 1);
        }
    }

    #[test]
    fn flagging_cycles_and_prevents_reveal() {
        let mut board = Board::new(Difficulty::Beginner, Some(42));
        board.reveal(0, 0);
        let hidden = (0..board.rows)
            .flat_map(|r| (0..board.cols).map(move |c| (r, c)))
            .find(|&(r, c)| board.cell(r, c).state == CellState::Hidden)
            .unwrap();

        assert!(board.toggle_flag(hidden.0, hidden.1, false));
        assert_eq!(board.cell(hidden.0, hidden.1).state, CellState::Flagged);
        assert_eq!(board.bombs_remaining(), 9);
        assert_eq!(board.reveal(hidden.0, hidden.1), RevealOutcome::NoOp);

        // With question marks enabled, Flagged -> Question -> Hidden.
        assert!(board.toggle_flag(hidden.0, hidden.1, true));
        assert_eq!(board.cell(hidden.0, hidden.1).state, CellState::Question);
        assert_eq!(board.bombs_remaining(), 10);

        assert!(board.toggle_flag(hidden.0, hidden.1, true));
        assert_eq!(board.cell(hidden.0, hidden.1).state, CellState::Hidden);
    }

    #[test]
    fn chording_opens_neighbors_when_flags_match() {
        let mut board = Board::new(Difficulty::Beginner, Some(7));
        board.reveal(4, 4);
        // Find a revealed numbered tile and flag all its actual bomb neighbors.
        let numbered = (0..board.rows)
            .flat_map(|r| (0..board.cols).map(move |c| (r, c)))
            .find(|&(r, c)| {
                let cell = board.cell(r, c);
                cell.state == CellState::Revealed
                    && cell.adjacent > 0
                    && board
                        .neighbors(r, c)
                        .any(|(nr, nc)| !board.cell(nr, nc).has_bomb && board.cell(nr, nc).state == CellState::Hidden)
            })
            .expect("expected a frontier numbered tile");

        let bomb_neighbors: Vec<_> = board
            .neighbors(numbered.0, numbered.1)
            .filter(|&(nr, nc)| board.cell(nr, nc).has_bomb)
            .collect();
        for (br, bc) in bomb_neighbors {
            board.toggle_flag(br, bc, false);
        }
        assert!(board.can_chord(numbered.0, numbered.1));
        let outcome = board.chord(numbered.0, numbered.1);
        assert!(matches!(outcome, RevealOutcome::Opened { count } | RevealOutcome::Won { count } if count >= 1));
    }

    #[test]
    fn winning_auto_flags_all_bombs() {
        let board = Board::demo_won(Difficulty::Beginner, 99);
        assert_eq!(board.phase, GamePhase::Won);
        assert_eq!(board.bombs_remaining(), 0);
        assert_eq!(board.progress(), 1.0);
        for cell in &board.cells {
            if cell.has_bomb {
                assert_eq!(cell.state, CellState::Flagged);
            } else {
                assert_eq!(cell.state, CellState::Revealed);
            }
        }
    }

    #[test]
    fn hitting_a_bomb_reveals_all_bombs_and_marks_misflags() {
        let board = Board::demo_lost(Difficulty::Beginner, 13);
        assert_eq!(board.phase, GamePhase::Lost);
        assert!(board.last_blast.is_some());
        assert_eq!(board.cells.iter().filter(|c| c.detonated).count(), 1);
        assert!(board.cells.iter().any(|c| c.misflagged));
        for cell in &board.cells {
            if cell.has_bomb && cell.state != CellState::Flagged {
                assert_eq!(cell.state, CellState::Revealed);
            }
        }
    }

    #[test]
    fn subset_hint_deduces_safe_and_bomb_cells() {
        // Construct a classic 1-2-1 pattern on a small board to test exact deduction.
        let mut board = Board::new(Difficulty::Custom { cols: 6, rows: 6, bombs: 2 }, Some(1));
        board.phase = GamePhase::Playing;
        board.cell_mut(0, 1).has_bomb = true;
        board.cell_mut(0, 3).has_bomb = true;
        board.recompute_adjacencies_and_3bv();

        // Reveal row 1: cells (1,1)=1, (1,2)=2, (1,3)=1, and (1,0)=1, (1,4)=1, plus all of row 2..5.
        for r in 1..6 {
            for c in 0..6 {
                board.cell_mut(r, c).state = CellState::Revealed;
                board.revealed_safe += 1;
            }
        }
        // Cell (1, 0) has adjacent=1 with unknown {(0,0), (0,1)}.
        // Cell (1, 1) has adjacent=1 with unknown {(0,0), (0,1), (0,2)}!
        // Subset {(0,0), (0,1)} ⊂ {(0,0), (0,1), (0,2)} with rem_diff = 0 => (0,2) is 100% safe!
        let hint = board.compute_hint().expect("should deduce safe tile");
        assert_eq!(hint.kind, HintKind::Safe);
        assert_eq!((hint.row, hint.col), (0, 2));
    }
}
