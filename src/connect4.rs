//! Seven columns, six rows, gravity, four directions, and a tactical CPU.
pub const COLS: usize = 7;
pub const ROWS: usize = 6;
#[derive(Clone, Debug)]
pub struct Connect4 {
    pub board: [u8; 42],
    pub turn: u8,
    pub winner: Option<u8>,
    pub draw: bool,
    pub ply: u8,
    pub last: Option<usize>,
    pub line: Vec<usize>,
}
impl Default for Connect4 {
    fn default() -> Self {
        Self::new()
    }
}
impl Connect4 {
    pub fn new() -> Self {
        Self {
            board: [0; 42],
            turn: 1,
            winner: None,
            draw: false,
            ply: 0,
            last: None,
            line: vec![],
        }
    }
    pub fn legal(&self) -> Vec<usize> {
        if self.winner.is_some() || self.draw {
            vec![]
        } else {
            [3, 2, 4, 1, 5, 0, 6]
                .into_iter()
                .filter(|&c| self.board[c] == 0)
                .collect()
        }
    }
    pub fn landing(&self, col: usize) -> Option<usize> {
        if col >= COLS {
            return None;
        }
        (0..ROWS)
            .rev()
            .map(|r| r * COLS + col)
            .find(|&s| self.board[s] == 0)
    }
    fn drop_raw(&mut self, col: usize) -> Option<usize> {
        let s = self.landing(col)?;
        let side = self.turn;
        self.board[s] = side;
        self.ply += 1;
        self.last = Some(s);
        let (x, y) = ((s % COLS) as i32, (s / COLS) as i32);
        for (dx, dy) in [(1, 0), (0, 1), (1, 1), (1, -1)] {
            let mut line = vec![s];
            for sign in [-1, 1] {
                let (mut nx, mut ny) = (x + dx * sign, y + dy * sign);
                while (0..COLS as i32).contains(&nx) && (0..ROWS as i32).contains(&ny) {
                    let c = ny as usize * COLS + nx as usize;
                    if self.board[c] != side {
                        break;
                    }
                    line.push(c);
                    nx += dx * sign;
                    ny += dy * sign;
                }
            }
            if line.len() >= 4 {
                self.winner = Some(side);
                self.line = line;
                break;
            }
        }
        self.draw = self.winner.is_none() && self.ply == 42;
        self.turn = 3 - side;
        Some(s)
    }
    pub fn play(&mut self, col: usize) -> Option<usize> {
        if !self.legal().contains(&col) {
            None
        } else {
            self.drop_raw(col)
        }
    }
    pub fn winning_columns(&self, side: u8) -> Vec<usize> {
        if self.winner.is_some() || self.draw {
            return vec![];
        }
        (0..COLS)
            .filter(|&c| {
                let mut next = self.clone();
                next.turn = side;
                next.drop_raw(c).is_some() && next.winner == Some(side)
            })
            .collect()
    }
    fn evaluate(&self, side: u8) -> i32 {
        let mut v = 0;
        for r in 0..ROWS {
            if self.board[r * COLS + 3] == side {
                v += 6;
            }
        }
        for y in 0..ROWS {
            for x in 0..COLS {
                for (dx, dy) in [(1, 0), (0, 1), (1, 1), (1, -1)] {
                    let (ex, ey) = (x as i32 + dx * 3, y as i32 + dy * 3);
                    if !(0..COLS as i32).contains(&ex) || !(0..ROWS as i32).contains(&ey) {
                        continue;
                    }
                    let (mut mine, mut theirs) = (0, 0);
                    for i in 0..4 {
                        let c = (y as i32 + dy * i) as usize * COLS + (x as i32 + dx * i) as usize;
                        mine += i32::from(self.board[c] == side);
                        theirs += i32::from(self.board[c] == 3 - side);
                    }
                    if theirs == 0 {
                        v += [0, 1, 12, 100, 10000][mine as usize];
                    }
                    if mine == 0 {
                        v -= [0, 1, 14, 130, 10000][theirs as usize];
                    }
                }
            }
        }
        v
    }
    fn search(&self, depth: u8, mut alpha: i32, beta: i32) -> i32 {
        if let Some(w) = self.winner {
            return if w == self.turn {
                100000 + depth as i32
            } else {
                -100000 - depth as i32
            };
        }
        if self.draw {
            return 0;
        }
        if depth == 0 {
            return self.evaluate(self.turn);
        }
        for c in self.legal() {
            let mut next = self.clone();
            next.drop_raw(c);
            let score = -next.search(depth - 1, -beta, -alpha);
            if score >= beta {
                return beta;
            }
            alpha = alpha.max(score);
        }
        alpha
    }
    pub fn cpu(&self) -> Option<usize> {
        if let Some(&c) = self.winning_columns(self.turn).first() {
            return Some(c);
        }
        if let Some(&c) = self.winning_columns(3 - self.turn).first() {
            return Some(c);
        }
        let mut best = None;
        let mut alpha = -1000000;
        for c in self.legal() {
            let mut next = self.clone();
            next.drop_raw(c);
            let score = -next.search(5, -1000000, -alpha);
            if score > alpha {
                alpha = score;
                best = Some(c);
            }
        }
        best
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gravity_full_and_invalid_are_atomic() {
        let mut g = Connect4::new();
        assert_eq!(g.play(8), None);
        for _ in 0..6 {
            assert!(g.play(0).is_some());
        }
        let old = g.board;
        assert_eq!(g.play(0), None);
        assert_eq!(g.board, old);
        assert_eq!(g.ply, 6);
    }
    #[test]
    fn four_in_all_directions() {
        for (dx, dy, start) in [(1, 0, 35), (0, 1, 0), (1, 1, 0), (1, -1, 35)] {
            let mut g = Connect4::new();
            for i in 0..3 {
                g.board[(start + dx * i + dy * i * 7) as usize] = 1;
            }
            let last = (start + dx * 3 + dy * 3 * 7) as usize;
            for r in last / 7 + 1..6 {
                g.board[r * 7 + last % 7] = 2;
            }
            g.turn = 1;
            g.drop_raw(last % 7);
            assert_eq!(g.winner, Some(1));
            assert!(g.line.len() >= 4);
        }
    }
    #[test]
    fn cpu_wins_and_blocks() {
        let mut g = Connect4::new();
        for c in [0, 6, 1, 6, 2, 5] {
            g.play(c);
        }
        assert_eq!(g.cpu(), Some(3));
        assert_eq!(g.winning_columns(1), vec![3]);
        g.turn = 2;
        assert_eq!(g.cpu(), Some(3));
    }
    #[test]
    fn complete_cpu_games_are_legal() {
        for opening in 0..7 {
            let mut g = Connect4::new();
            g.play(opening);
            while g.winner.is_none() && !g.draw {
                let c = g.cpu().unwrap();
                assert!(g.play(c).is_some());
                assert!(g.ply <= 42);
            }
            assert!(g.winner.is_some() || g.draw);
        }
    }
    #[test]
    fn full_board_without_four_is_a_draw() {
        let mut g = Connect4::new();
        let rows = [
            "1212121", "1212121", "2121212", "2121212", "1212121", "2121212",
        ];
        for (y, row) in rows.iter().enumerate() {
            for (x, b) in row.bytes().enumerate() {
                g.board[y * 7 + x] = b - b'0';
            }
        }
        g.board[1] = 0;
        g.ply = 41;
        g.turn = 2;
        assert_eq!(g.play(1), Some(1));
        assert!(g.draw);
        assert_eq!(g.winner, None);
        assert!(g.legal().is_empty());
    }
    #[test]
    fn no_actions_after_win() {
        let mut g = Connect4::new();
        for c in [0, 1, 0, 1, 0, 1, 0] {
            g.play(c);
        }
        assert_eq!(g.winner, Some(1));
        assert!(g.cpu().is_none());
        assert_eq!(g.play(2), None);
    }
}
