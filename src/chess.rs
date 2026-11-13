//! Complete standard move legality, draw detection, and a small deterministic CPU search.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    pub from: usize,
    pub to: usize,
    pub promotion: i8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Won(i8),
    Draw(&'static str),
}
#[derive(Clone)]
pub struct Chess {
    pub board: [i8; 64],
    pub turn: i8,
    pub castle: u8,
    pub ep: Option<usize>,
    pub halfmove: u16,
    pub ply: u32,
    pub last: Option<Move>,
    pub outcome: Option<Outcome>,
    positions: Vec<String>,
}
fn square(x: i32, y: i32) -> Option<usize> {
    if (0..8).contains(&x) && (0..8).contains(&y) {
        Some((y * 8 + x) as usize)
    } else {
        None
    }
}
pub fn name(s: usize) -> String {
    format!("{}{}", (b'a' + (s % 8) as u8) as char, 8 - s / 8)
}
impl Default for Chess {
    fn default() -> Self {
        Self::new()
    }
}
impl Chess {
    pub fn new() -> Self {
        let mut board = [0; 64];
        let back = [4, 2, 3, 5, 6, 3, 2, 4];
        for x in 0..8 {
            board[x] = -back[x];
            board[8 + x] = -1;
            board[48 + x] = 1;
            board[56 + x] = back[x];
        }
        let mut g = Self {
            board,
            turn: 1,
            castle: 15,
            ep: None,
            halfmove: 0,
            ply: 0,
            last: None,
            outcome: None,
            positions: vec![],
        };
        g.positions.push(g.key());
        g
    }
    fn key(&self) -> String {
        // En passant affects repetition only when a legal en passant capture exists.
        let ep = self.ep.filter(|&s| {
            self.legal_raw()
                .iter()
                .any(|m| m.to == s && self.board[m.from].abs() == 1 && m.from % 8 != s % 8)
        });
        format!("{:?}/{}/{}/{:?}", self.board, self.turn, self.castle, ep)
    }
    pub fn attacked(&self, s: usize, by: i8) -> bool {
        let (sx, sy) = ((s % 8) as i32, (s / 8) as i32);
        for from in 0..64 {
            let p = self.board[from];
            if p.signum() != by {
                continue;
            }
            let (x, y) = ((from % 8) as i32, (from / 8) as i32);
            let (dx, dy) = (sx - x, sy - y);
            match p.abs() {
                1 => {
                    if dx.abs() == 1 && dy == -by as i32 {
                        return true;
                    }
                }
                2 => {
                    if (dx.abs() == 1 && dy.abs() == 2) || (dx.abs() == 2 && dy.abs() == 1) {
                        return true;
                    }
                }
                6 => {
                    if dx.abs().max(dy.abs()) == 1 {
                        return true;
                    }
                }
                kind => {
                    let diagonal = dx.abs() == dy.abs() && dx != 0;
                    let straight = (dx == 0) != (dy == 0);
                    if !((kind == 3 && diagonal)
                        || (kind == 4 && straight)
                        || (kind == 5 && (diagonal || straight)))
                    {
                        continue;
                    }
                    let (mx, my) = (dx.signum(), dy.signum());
                    let (mut nx, mut ny) = (x + mx, y + my);
                    let mut clear = true;
                    while nx != sx || ny != sy {
                        if self.board[square(nx, ny).unwrap()] != 0 {
                            clear = false;
                            break;
                        }
                        nx += mx;
                        ny += my;
                    }
                    if clear {
                        return true;
                    }
                }
            }
        }
        false
    }
    pub fn in_check(&self, side: i8) -> bool {
        self.board
            .iter()
            .position(|&p| p == side * 6)
            .is_none_or(|s| self.attacked(s, -side))
    }
    fn pseudo(&self) -> Vec<Move> {
        let mut out = vec![];
        let mut add = |from: usize, to: usize| {
            if self.board[from].abs() == 1 && (to / 8 == 0 || to / 8 == 7) {
                for promotion in [5, 4, 3, 2] {
                    out.push(Move {
                        from,
                        to,
                        promotion,
                    });
                }
            } else {
                out.push(Move {
                    from,
                    to,
                    promotion: 0,
                });
            }
        };
        for from in 0..64 {
            let p = self.board[from];
            if p.signum() != self.turn {
                continue;
            }
            let (x, y) = ((from % 8) as i32, (from / 8) as i32);
            if p.abs() == 1 {
                let dy = -self.turn as i32;
                if let Some(to) = square(x, y + dy).filter(|&t| self.board[t] == 0) {
                    add(from, to);
                    if y == if self.turn == 1 { 6 } else { 1 }
                        && let Some(t) = square(x, y + dy * 2).filter(|&t| self.board[t] == 0)
                    {
                        add(from, t);
                    }
                }
                for dx in [-1, 1] {
                    if let Some(to) = square(x + dx, y + dy)
                        && (self.board[to].signum() == -self.turn && self.board[to].abs() != 6
                            || self.ep == Some(to))
                    {
                        add(from, to);
                    }
                }
            } else {
                let dirs: &[(i32, i32)] = match p.abs() {
                    2 => &[
                        (1, 2),
                        (2, 1),
                        (-1, 2),
                        (-2, 1),
                        (1, -2),
                        (2, -1),
                        (-1, -2),
                        (-2, -1),
                    ],
                    3 => &[(1, 1), (1, -1), (-1, 1), (-1, -1)],
                    4 => &[(1, 0), (-1, 0), (0, 1), (0, -1)],
                    _ => &[
                        (1, 1),
                        (1, -1),
                        (-1, 1),
                        (-1, -1),
                        (1, 0),
                        (-1, 0),
                        (0, 1),
                        (0, -1),
                    ],
                };
                for &(dx, dy) in dirs {
                    let (mut nx, mut ny) = (x + dx, y + dy);
                    while let Some(to) = square(nx, ny) {
                        let target = self.board[to];
                        if target.signum() == self.turn || target.abs() == 6 {
                            break;
                        }
                        add(from, to);
                        if target != 0 || p.abs() == 2 || p.abs() == 6 {
                            break;
                        }
                        nx += dx;
                        ny += dy;
                    }
                }
                if p.abs() == 6 {
                    let (home, mask) = if self.turn == 1 { (60, 1) } else { (4, 4) };
                    if from == home && !self.in_check(self.turn) {
                        if self.castle & mask != 0
                            && self.board[home + 3] == self.turn * 4
                            && self.board[home + 1] == 0
                            && self.board[home + 2] == 0
                            && !self.attacked(home + 1, -self.turn)
                            && !self.attacked(home + 2, -self.turn)
                        {
                            add(from, home + 2);
                        }
                        if self.castle & (mask * 2) != 0
                            && self.board[home - 4] == self.turn * 4
                            && (1..=3).all(|n| self.board[home - n] == 0)
                            && !self.attacked(home - 1, -self.turn)
                            && !self.attacked(home - 2, -self.turn)
                        {
                            add(from, home - 2);
                        }
                    }
                }
            }
        }
        out
    }
    fn step(&mut self, m: Move) {
        let piece = self.board[m.from];
        let target = self.board[m.to];
        if piece.abs() == 1 && self.ep == Some(m.to) && target == 0 && m.from % 8 != m.to % 8 {
            self.board[(m.to as i32 + self.turn as i32 * 8) as usize] = 0;
        }
        self.board[m.from] = 0;
        self.board[m.to] = if m.promotion != 0 {
            m.promotion * self.turn
        } else {
            piece
        };
        if piece.abs() == 6 {
            self.castle &= if self.turn == 1 { 12 } else { 3 };
            if m.from.abs_diff(m.to) == 2 {
                let (rook_from, rook_to) = if m.to > m.from {
                    (m.from + 3, m.from + 1)
                } else {
                    (m.from - 4, m.from - 1)
                };
                self.board[rook_to] = self.board[rook_from];
                self.board[rook_from] = 0;
            }
        }
        for (s, mask) in [(63, 1), (56, 2), (7, 4), (0, 8)] {
            if m.from == s || m.to == s {
                self.castle &= !mask;
            }
        }
        self.ep = if piece.abs() == 1 && m.from.abs_diff(m.to) == 16 {
            Some((m.from + m.to) / 2)
        } else {
            None
        };
        self.halfmove = if piece.abs() == 1 || target != 0 {
            0
        } else {
            self.halfmove + 1
        };
        self.turn = -self.turn;
        self.ply += 1;
        self.last = Some(m);
    }
    fn legal_raw(&self) -> Vec<Move> {
        self.pseudo()
            .into_iter()
            .filter(|&m| {
                let mut next = self.clone();
                next.step(m);
                !next.in_check(self.turn)
            })
            .collect()
    }
    pub fn legal(&self) -> Vec<Move> {
        if self.outcome.is_some() {
            vec![]
        } else {
            self.legal_raw()
        }
    }
    pub fn play(&mut self, m: Move) -> bool {
        if !self.legal().contains(&m) {
            return false;
        }
        self.step(m);
        self.positions.push(self.key());
        self.adjudicate();
        true
    }
    fn dead_material(&self) -> bool {
        let pieces: Vec<_> = self
            .board
            .iter()
            .enumerate()
            .filter(|&(_, p)| *p != 0 && p.abs() != 6)
            .collect();
        if pieces.is_empty() {
            return true;
        }
        if pieces.len() == 1 && [2, 3].contains(&pieces[0].1.abs()) {
            return true;
        }
        pieces.iter().all(|(_, p)| p.abs() == 3)
            && pieces
                .iter()
                .map(|(s, _)| (s % 8 + s / 8) % 2)
                .all(|c| c == (pieces[0].0 % 8 + pieces[0].0 / 8) % 2)
    }
    fn adjudicate(&mut self) {
        if self.legal_raw().is_empty() {
            self.outcome = Some(if self.in_check(self.turn) {
                Outcome::Won(-self.turn)
            } else {
                Outcome::Draw("Stalemate")
            });
        } else if self.dead_material() {
            self.outcome = Some(Outcome::Draw("Insufficient material"));
        } else if self.halfmove >= 100 {
            self.outcome = Some(Outcome::Draw("50-move rule"));
        } else if self
            .positions
            .iter()
            .filter(|&k| Some(k) == self.positions.last())
            .count()
            >= 3
        {
            self.outcome = Some(Outcome::Draw("Threefold repetition"));
        }
    }
    fn value(&self) -> i32 {
        let mut result = 0;
        for (s, &p) in self.board.iter().enumerate() {
            if p == 0 {
                continue;
            }
            let v = [0, 100, 320, 335, 500, 900, 0][p.unsigned_abs() as usize];
            let center = 7 - ((s % 8) as i32 * 2 - 7).abs() - ((s / 8) as i32 * 2 - 7).abs();
            let development = if p.abs() == 1 {
                if p > 0 { 6 - s / 8 } else { s / 8 - 1 }
            } else {
                0
            };
            result += p.signum() as i32 * (v + center * 3 + development as i32 * 7);
        }
        result * self.turn as i32
    }
    fn search(&self, depth: u8, mut alpha: i32, beta: i32, nodes: &mut u32) -> i32 {
        *nodes += 1;
        let mut moves = self.legal_raw();
        if moves.is_empty() {
            return if self.in_check(self.turn) {
                -100000 + self.ply as i32
            } else {
                0
            };
        }
        if depth == 0 || *nodes > 18000 {
            return self.value();
        }
        moves.sort_by_key(|m| -self.board[m.to].abs());
        for m in moves {
            let mut g = self.clone();
            g.step(m);
            let score = -g.search(depth - 1, -beta, -alpha, nodes);
            if score >= beta {
                return beta;
            }
            alpha = alpha.max(score);
        }
        alpha
    }
    pub fn cpu(&self) -> Option<Move> {
        let mut moves = self.legal();
        moves.sort_by_key(|m| -self.board[m.to].abs());
        let mut best = None;
        let mut score = -1000000;
        let mut nodes = 0;
        for m in moves {
            let mut g = self.clone();
            g.step(m);
            let v = -g.search(2, -1000000, -score, &mut nodes);
            if v > score {
                score = v;
                best = Some(m);
            }
        }
        best
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn perft(g: &Chess, d: u8) -> u64 {
        if d == 0 {
            return 1;
        }
        g.legal_raw()
            .iter()
            .map(|&m| {
                let mut n = g.clone();
                n.step(m);
                perft(&n, d - 1)
            })
            .sum()
    }
    fn empty() -> Chess {
        let mut g = Chess::new();
        g.board = [0; 64];
        g.castle = 0;
        g.positions.clear();
        g
    }
    fn go(g: &mut Chess, a: usize, b: usize) {
        assert!(g.play(Move {
            from: a,
            to: b,
            promotion: 0
        }));
    }
    #[test]
    fn kiwipete_castling_perft() {
        let mut g = empty();
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R";
        let mut s = 0;
        for c in fen.chars() {
            if c == '/' {
                continue;
            }
            if let Some(n) = c.to_digit(10) {
                s += n as usize;
            } else {
                let k = match c.to_ascii_lowercase() {
                    'p' => 1,
                    'n' => 2,
                    'b' => 3,
                    'r' => 4,
                    'q' => 5,
                    _ => 6,
                };
                g.board[s] = if c.is_uppercase() { k } else { -k };
                s += 1;
            }
        }
        g.castle = 15;
        assert_eq!(perft(&g, 1), 48);
        assert_eq!(perft(&g, 2), 2039);
        assert_eq!(perft(&g, 3), 97862);
    }
    #[test]
    fn standard_perft() {
        let g = Chess::new();
        assert_eq!(perft(&g, 1), 20);
        assert_eq!(perft(&g, 2), 400);
        assert_eq!(perft(&g, 3), 8902);
    }
    #[test]
    fn fools_mate() {
        let mut g = Chess::new();
        go(&mut g, 53, 45);
        go(&mut g, 12, 28);
        go(&mut g, 54, 38);
        go(&mut g, 3, 39);
        assert_eq!(g.outcome, Some(Outcome::Won(-1)));
    }
    #[test]
    fn en_passant_and_discovered_check() {
        let mut g = empty();
        g.board[60] = 6;
        g.board[4] = -6;
        g.board[28] = 1;
        g.board[11] = -1;
        g.turn = -1;
        go(&mut g, 11, 27);
        go(&mut g, 28, 19);
        assert_eq!(g.board[27], 0);
        assert_eq!(g.board[19], 1);
        let mut g = empty();
        g.board[28] = 6;
        g.board[0] = -6;
        g.board[29] = 1;
        g.board[30] = -1;
        g.board[31] = -4;
        g.ep = Some(22);
        assert!(!g.legal().iter().any(|m| m.from == 29 && m.to == 22));
    }
    #[test]
    fn castle_through_check_and_rook_move() {
        let mut g = empty();
        g.board[60] = 6;
        g.board[4] = -6;
        g.board[63] = 4;
        g.castle = 1;
        assert!(g.legal().iter().any(|m| m.to == 62));
        g.board[5] = -4;
        assert!(!g.legal().iter().any(|m| m.from == 60 && m.to == 62));
        g.board[5] = 0;
        go(&mut g, 60, 62);
        assert_eq!(g.board[61], 4);
        assert_eq!(g.board[63], 0);
    }
    #[test]
    fn promotions_and_stalemate() {
        let mut g = empty();
        g.board[63] = 6;
        g.board[7] = -6;
        g.board[8] = 1;
        assert_eq!(
            g.legal()
                .iter()
                .filter(|m| m.from == 8 && m.to == 0)
                .count(),
            4
        );
        assert!(g.play(Move {
            from: 8,
            to: 0,
            promotion: 2
        }));
        assert_eq!(g.board[0], 2);
        let mut g = empty();
        g.board[0] = -6;
        g.board[18] = 6;
        g.board[17] = 5;
        g.turn = -1;
        g.adjudicate();
        assert_eq!(g.outcome, Some(Outcome::Draw("Stalemate")));
    }
    #[test]
    fn repetition_draw() {
        let mut g = Chess::new();
        for _ in 0..2 {
            go(&mut g, 62, 45);
            go(&mut g, 6, 21);
            go(&mut g, 45, 62);
            go(&mut g, 21, 6);
        }
        assert_eq!(g.outcome, Some(Outcome::Draw("Threefold repetition")));
    }
    #[test]
    fn cpu_is_legal_and_finds_mate() {
        let mut g = Chess::new();
        go(&mut g, 53, 45);
        go(&mut g, 12, 28);
        go(&mut g, 54, 38);
        let m = g.cpu().unwrap();
        assert!(g.play(m));
        assert_eq!(g.outcome, Some(Outcome::Won(-1)));
    }
}
