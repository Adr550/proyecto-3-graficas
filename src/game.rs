//! The two-player Quoridor rules, independent of rendering. The application controls only player 0.
use std::collections::VecDeque;
pub const SIZE: usize = 9;
pub const CELLS: usize = 81;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}
impl Axis {
    pub fn other(self) -> Self {
        if self == Self::Horizontal {
            Self::Vertical
        } else {
            Self::Horizontal
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wall {
    pub x: u8,
    pub y: u8,
    pub axis: Axis,
    pub owner: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Move(usize),
    Place { x: u8, y: u8, axis: Axis },
}
#[derive(Clone, Debug)]
pub struct Game {
    pub pawns: [usize; 2],
    pub walls: Vec<Wall>,
    pub left: [u8; 2],
    pub turn: usize,
    pub winner: Option<usize>,
    pub ply: u32,
    pub last: [Option<usize>; 2],
    pub log: VecDeque<String>,
}
pub fn xy(cell: usize) -> (usize, usize) {
    (cell % SIZE, cell / SIZE)
}
pub fn coord(cell: usize) -> String {
    let (x, y) = xy(cell);
    format!("{}{}", (b'A' + x as u8) as char, y + 1)
}
pub fn neighbor(cell: usize, d: usize) -> Option<usize> {
    let (x, y) = xy(cell);
    match d {
        0 if y > 0 => Some(cell - SIZE),
        1 if x + 1 < SIZE => Some(cell + 1),
        2 if y + 1 < SIZE => Some(cell + SIZE),
        3 if x > 0 => Some(cell - 1),
        _ => None,
    }
}
pub fn goal(player: usize, cell: usize) -> bool {
    cell / SIZE == if player == 0 { 0 } else { 8 }
}
impl Game {
    pub fn new() -> Self {
        let mut g = Self {
            pawns: [76, 4],
            walls: vec![],
            left: [10, 10],
            turn: 0,
            winner: None,
            ply: 0,
            last: [None, None],
            log: VecDeque::new(),
        };
        g.note("Welcome! Fly to any planet on the far blue edge.");
        g
    }
    pub fn note(&mut self, s: impl Into<String>) {
        self.log.push_front(s.into());
        self.log.truncate(5);
    }
    pub fn blocked(&self, a: usize, b: usize) -> bool {
        let (ax, ay) = xy(a);
        let (bx, by) = xy(b);
        if ax.abs_diff(bx) + ay.abs_diff(by) != 1 {
            return true;
        }
        self.walls.iter().any(|w| match w.axis {
            Axis::Horizontal => {
                ax == bx
                    && ay.min(by) == w.y as usize
                    && (ax == w.x as usize || ax == w.x as usize + 1)
            }
            Axis::Vertical => {
                ay == by
                    && ax.min(bx) == w.x as usize
                    && (ay == w.y as usize || ay == w.y as usize + 1)
            }
        })
    }
    pub fn moves(&self, player: usize) -> Vec<usize> {
        let source = self.pawns[player];
        let other = self.pawns[1 - player];
        let mut out = Vec::with_capacity(5);
        for d in 0..4 {
            let Some(next) = neighbor(source, d) else {
                continue;
            };
            if self.blocked(source, next) {
                continue;
            }
            if next != other {
                out.push(next);
                continue;
            }
            let straight = neighbor(other, d).filter(|&b| !self.blocked(other, b));
            if let Some(b) = straight {
                out.push(b);
            } else {
                for side in [(d + 1) % 4, (d + 3) % 4] {
                    if let Some(b) = neighbor(other, side).filter(|&b| !self.blocked(other, b)) {
                        out.push(b);
                    }
                }
            }
        }
        out.sort_unstable();
        out.dedup();
        out
    }
    /// A wall must leave a route even if the other pawn is ignored, as in the published rules.
    pub fn distances(&self, player: usize) -> [u16; CELLS] {
        let mut dist = [u16::MAX; CELLS];
        let mut queue = [0usize; CELLS];
        let (mut head, mut tail) = (0, 0);
        for x in 0..SIZE {
            let c = if player == 0 { x } else { 72 + x };
            dist[c] = 0;
            queue[tail] = c;
            tail += 1;
        }
        while head < tail {
            let a = queue[head];
            head += 1;
            for d in 0..4 {
                if let Some(b) = neighbor(a, d)
                    && dist[b] == u16::MAX
                    && !self.blocked(a, b)
                {
                    dist[b] = dist[a] + 1;
                    queue[tail] = b;
                    tail += 1;
                }
            }
        }
        dist
    }
    pub fn distance(&self, player: usize) -> u16 {
        self.distances(player)[self.pawns[player]]
    }
    pub fn wall_valid(&self, x: u8, y: u8, axis: Axis) -> Result<(), &'static str> {
        if x >= 8 || y >= 8 {
            return Err("Choose an intersection inside the board.");
        }
        for w in &self.walls {
            if w.x == x && w.y == y {
                return Err("Barriers cannot cross or overlap.");
            }
            let overlap = match axis {
                Axis::Horizontal => w.axis == axis && w.y == y && w.x.abs_diff(x) <= 1,
                Axis::Vertical => w.axis == axis && w.x == x && w.y.abs_diff(y) <= 1,
            };
            if overlap {
                return Err("That barrier overlaps an existing one.");
            }
        }
        let mut next = self.clone();
        next.walls.push(Wall {
            x,
            y,
            axis,
            owner: self.turn,
        });
        if next.distance(0) == u16::MAX || next.distance(1) == u16::MAX {
            return Err("Keep a route to the finish for both ships.");
        }
        Ok(())
    }
    pub fn validate(&self, a: Action) -> Result<(), &'static str> {
        if self.winner.is_some() {
            return Err("Match complete. Start a rematch!");
        }
        match a {
            Action::Move(to) => {
                if to >= CELLS || !self.moves(self.turn).contains(&to) {
                    return Err("Choose a glowing neighbor. Barriers block travel.");
                }
            }
            Action::Place { x, y, axis } => {
                if self.left[self.turn] == 0 {
                    return Err("No barriers left. Move your ship instead.");
                }
                self.wall_valid(x, y, axis)?;
            }
        }
        Ok(())
    }
    pub fn apply(&mut self, a: Action) -> Result<(), &'static str> {
        self.validate(a)?;
        let player = self.turn;
        let who = if player == 0 { "You" } else { "Truffle" };
        match a {
            Action::Move(to) => {
                self.last[player] = Some(self.pawns[player]);
                self.pawns[player] = to;
                self.note(format!("{who} flew to {}.", coord(to)));
                if goal(player, to) {
                    self.winner = Some(player);
                    self.note(if player == 0 {
                        "Finish! You won the race!"
                    } else {
                        "Truffle reached the finish. Try a rematch!"
                    });
                }
            }
            Action::Place { x, y, axis } => {
                self.walls.push(Wall {
                    x,
                    y,
                    axis,
                    owner: player,
                });
                self.left[player] -= 1;
                self.note(format!(
                    "{who} placed a {} barrier.",
                    if axis == Axis::Horizontal {
                        "horizontal"
                    } else {
                        "vertical"
                    }
                ));
            }
        }
        self.ply += 1;
        if self.winner.is_none() {
            self.turn = 1 - self.turn;
        }
        Ok(())
    }
}
impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opening_and_turns() {
        let mut g = Game::new();
        assert_eq!(g.moves(0), vec![67, 75, 77]);
        assert_eq!(g.distance(0), 8);
        g.apply(Action::Move(67)).unwrap();
        assert_eq!(g.turn, 1);
        assert_eq!(g.pawns, [67, 4]);
    }
    #[test]
    fn walls_block_exact_two_edges() {
        let mut g = Game::new();
        g.apply(Action::Place {
            x: 3,
            y: 6,
            axis: Axis::Horizontal,
        })
        .unwrap();
        assert!(g.blocked(57, 66));
        assert!(g.blocked(58, 67));
        assert!(!g.blocked(59, 68));
        assert_eq!(g.left, [9, 10]);
    }
    #[test]
    fn straight_jump_is_mandatory_instead_of_diagonal() {
        let mut g = Game::new();
        g.pawns = [40, 31];
        let moves = g.moves(0);
        assert!(moves.contains(&22));
        assert!(!moves.contains(&30));
        assert!(!moves.contains(&32));
        assert!(!moves.contains(&31));
    }
    #[test]
    fn blocked_jump_allows_side_steps() {
        let mut g = Game::new();
        g.pawns = [40, 31];
        g.walls.push(Wall {
            x: 3,
            y: 2,
            axis: Axis::Horizontal,
            owner: 0,
        });
        let moves = g.moves(0);
        assert!(moves.contains(&30));
        assert!(moves.contains(&32));
        assert!(!moves.contains(&22));
    }
    #[test]
    fn board_edge_allows_side_steps() {
        let mut g = Game::new();
        g.pawns = [13, 4];
        assert!(g.moves(0).contains(&3));
        assert!(g.moves(0).contains(&5));
    }
    #[test]
    fn overlap_cross_and_touch_rules() {
        let mut g = Game::new();
        g.walls.push(Wall {
            x: 2,
            y: 3,
            axis: Axis::Horizontal,
            owner: 0,
        });
        assert!(g.wall_valid(2, 3, Axis::Vertical).is_err());
        assert!(g.wall_valid(3, 3, Axis::Horizontal).is_err());
        assert!(g.wall_valid(4, 3, Axis::Horizontal).is_ok());
        assert!(g.wall_valid(3, 3, Axis::Vertical).is_ok());
    }
    #[test]
    fn cannot_seal_a_player_in() {
        let mut g = Game::new();
        g.pawns[0] = 76;
        g.walls.push(Wall {
            x: 3,
            y: 7,
            axis: Axis::Horizontal,
            owner: 1,
        });
        g.walls.push(Wall {
            x: 2,
            y: 7,
            axis: Axis::Vertical,
            owner: 1,
        });
        assert!(g.wall_valid(4, 7, Axis::Vertical).is_err());
    }
    #[test]
    fn invalid_actions_are_atomic() {
        let mut g = Game::new();
        assert!(g.apply(Action::Move(0)).is_err());
        assert_eq!(g.turn, 0);
        assert_eq!(g.ply, 0);
        assert_eq!(g.left, [10, 10]);
    }
    #[test]
    fn reaching_any_far_cell_wins() {
        let mut g = Game::new();
        g.pawns[0] = 9;
        g.apply(Action::Move(0)).unwrap();
        assert_eq!(g.winner, Some(0));
        assert!(g.apply(Action::Move(1)).is_err());
    }
    #[test]
    fn barrier_inventory_is_enforced() {
        let mut g = Game::new();
        g.left[0] = 0;
        assert!(
            g.apply(Action::Place {
                x: 0,
                y: 0,
                axis: Axis::Horizontal
            })
            .is_err()
        );
        assert!(g.apply(Action::Move(67)).is_ok());
    }
}
