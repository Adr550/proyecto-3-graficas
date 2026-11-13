//! Blast Orbit: a real-time, one-human/one-CPU bomb arena with fair timed explosions.
use std::collections::VecDeque;
pub const W: usize = 13;
pub const H: usize = 11;
pub const N: usize = W * H;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Floor,
    Stone,
    Crate,
}
#[derive(Clone, Copy, Debug)]
pub struct Bomb {
    pub cell: usize,
    pub owner: usize,
    pub fuse: f32,
    pub range: usize,
}
#[derive(Clone, Debug)]
pub struct Flame {
    pub cell: usize,
    pub life: f32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    Win(usize),
    Draw,
}
pub struct Arena {
    pub tiles: [Tile; N],
    pub pawns: [usize; 2],
    pub alive: [bool; 2],
    pub bombs: Vec<Bomb>,
    pub flames: Vec<Flame>,
    pub pickups: [bool; N],
    pub range: [usize; 2],
    pub remaining: f32,
    pub outcome: Option<End>,
    pub destroyed: u32,
    cpu_clock: f32,
    cpu_bomb_clock: f32,
}
pub fn adjacent(c: usize) -> Vec<usize> {
    let (x, y) = (c % W, c / W);
    let mut v = vec![];
    if x > 0 {
        v.push(c - 1);
    }
    if x + 1 < W {
        v.push(c + 1);
    }
    if y > 0 {
        v.push(c - W);
    }
    if y + 1 < H {
        v.push(c + W);
    }
    v
}
impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}
impl Arena {
    pub fn new() -> Self {
        let mut tiles = [Tile::Floor; N];
        for (c, t) in tiles.iter_mut().enumerate() {
            let (x, y) = (c % W, c / W);
            *t = if x == 0 || y == 0 || x == W - 1 || y == H - 1 || x % 2 == 0 && y % 2 == 0 {
                Tile::Stone
            } else if (x * 17 + y * 31) % 7 < 5 {
                Tile::Crate
            } else {
                Tile::Floor
            };
        }
        let pawns = [W + 1, (H - 2) * W + W - 2];
        for c in [
            W + 1,
            W + 2,
            W * 2 + 1,
            W + 3,
            W * 3 + 1,
            (H - 2) * W + W - 2,
            (H - 2) * W + W - 3,
            (H - 3) * W + W - 2,
            (H - 2) * W + W - 4,
            (H - 4) * W + W - 2,
        ] {
            tiles[c] = Tile::Floor;
        }
        Self {
            tiles,
            pawns,
            alive: [true; 2],
            bombs: vec![],
            flames: vec![],
            pickups: [false; N],
            range: [3; 2],
            remaining: 120.,
            outcome: None,
            destroyed: 0,
            cpu_clock: 0.,
            cpu_bomb_clock: 0.,
        }
    }
    pub fn rays(&self, b: Bomb) -> Vec<usize> {
        let mut cells = vec![b.cell];
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (mut x, mut y) = ((b.cell % W) as i32, (b.cell / W) as i32);
            for _ in 0..b.range {
                x += dx;
                y += dy;
                if x < 0 || y < 0 || x >= W as i32 || y >= H as i32 {
                    break;
                }
                let c = y as usize * W + x as usize;
                if self.tiles[c] == Tile::Stone {
                    break;
                }
                cells.push(c);
                if self.tiles[c] == Tile::Crate {
                    break;
                }
            }
        }
        cells
    }
    pub fn walk(&mut self, who: usize, to: usize) -> bool {
        if self.outcome.is_some()
            || !self.alive[who]
            || !adjacent(self.pawns[who]).contains(&to)
            || self.tiles[to] != Tile::Floor
            || self.bombs.iter().any(|b| b.cell == to)
            || self.pawns[1 - who] == to
        {
            return false;
        }
        self.pawns[who] = to;
        if self.flames.iter().any(|f| f.cell == to) {
            self.alive[who] = false;
        }
        if self.pickups[to] {
            self.pickups[to] = false;
            self.range[who] = (self.range[who] + 1).min(6);
        }
        true
    }
    pub fn plant(&mut self, who: usize) -> bool {
        let cell = self.pawns[who];
        if self.outcome.is_some()
            || !self.alive[who]
            || self.bombs.iter().any(|b| b.cell == cell)
            || self.bombs.iter().filter(|b| b.owner == who).count() >= 2
        {
            return false;
        }
        self.bombs.push(Bomb {
            cell,
            owner: who,
            fuse: 2.2,
            range: self.range[who],
        });
        true
    }
    /// Earliest flame arrival, including chain reactions. Uses the same walls and ranges as real blasts.
    fn danger(&self, bombs: &[Bomb]) -> [f32; N] {
        let mut times: Vec<_> = bombs.iter().map(|b| b.fuse.max(0.)).collect();
        for _ in 0..bombs.len() {
            for (i, &b) in bombs.iter().enumerate() {
                for c in self.rays(b) {
                    for (j, other) in bombs.iter().enumerate() {
                        if other.cell == c {
                            times[j] = times[j].min(times[i]);
                        }
                    }
                }
            }
        }
        let mut danger = [f32::INFINITY; N];
        for (i, &b) in bombs.iter().enumerate() {
            for c in self.rays(b) {
                danger[c] = danger[c].min(times[i]);
            }
        }
        for f in &self.flames {
            danger[f.cell] = 0.;
        }
        danger
    }
    fn escape(&self, start: usize, bombs: &[Bomb]) -> Option<Vec<usize>> {
        let danger = self.danger(bombs);
        let mut q = VecDeque::from([(start, vec![])]);
        let mut seen = [false; N];
        seen[start] = true;
        while let Some((c, path)) = q.pop_front() {
            if danger[c].is_infinite() {
                return Some(path);
            }
            let arrival = (path.len() + 1) as f32 * 0.17;
            for next in adjacent(c) {
                if seen[next]
                    || self.tiles[next] != Tile::Floor
                    || self.pawns[0] == next
                    || bombs.iter().any(|b| b.cell == next)
                    || self.flames.iter().any(|f| f.cell == next)
                    || danger[next] <= arrival + 0.2
                {
                    continue;
                }
                seen[next] = true;
                let mut p = path.clone();
                p.push(next);
                q.push_back((next, p));
            }
        }
        None
    }
    fn cpu(&mut self) {
        let here = self.pawns[1];
        let danger = self.danger(&self.bombs);
        if !danger[here].is_infinite() {
            if let Some(path) = self.escape(here, &self.bombs)
                && let Some(&next) = path.first()
            {
                self.walk(1, next);
            }
            return;
        }
        let proposal = Bomb {
            cell: here,
            owner: 1,
            fuse: 2.2,
            range: self.range[1],
        };
        let rays = self.rays(proposal);
        let useful =
            rays.contains(&self.pawns[0]) || rays.iter().any(|&c| self.tiles[c] == Tile::Crate);
        let mut future = self.bombs.clone();
        future.push(proposal);
        if useful
            && self.cpu_bomb_clock <= 0.
            && self.escape(here, &future).is_some_and(|p| !p.is_empty())
            && self.plant(1)
        {
            self.cpu_bomb_clock = 0.8;
            return;
        }
        // Walk toward the rival or the nearest destructible obstacle, while avoiding all forecast blasts.
        let mut q = VecDeque::from([(here, None, 0usize)]);
        let mut seen = [false; N];
        seen[here] = true;
        let mut best = None;
        let mut best_score = i32::MIN;
        while let Some((c, first, dist)) = q.pop_front() {
            if let Some(step) = first {
                let dx = (c % W).abs_diff(self.pawns[0] % W);
                let dy = (c / W).abs_diff(self.pawns[0] / W);
                let crate_near = adjacent(c).iter().any(|&n| self.tiles[n] == Tile::Crate);
                let score = -(dx as i32 + dy as i32) * 3 - dist as i32
                    + if crate_near { 8 } else { 0 }
                    + if self.pickups[c] { 20 } else { 0 };
                if score > best_score {
                    best_score = score;
                    best = Some(step);
                }
            }
            for next in adjacent(c) {
                if !seen[next]
                    && self.tiles[next] == Tile::Floor
                    && next != self.pawns[0]
                    && danger[next].is_infinite()
                    && !self.bombs.iter().any(|b| b.cell == next)
                {
                    seen[next] = true;
                    q.push_back((next, first.or(Some(next)), dist + 1));
                }
            }
        }
        if let Some(c) = best {
            self.walk(1, c);
        }
    }
    pub fn update(&mut self, dt: f32) {
        self.advance(dt, true);
    }
    fn advance(&mut self, dt: f32, ai: bool) {
        if self.outcome.is_some() {
            return;
        }
        let dt = dt.clamp(0., 0.1);
        self.remaining = (self.remaining - dt).max(0.);
        self.cpu_clock += dt;
        self.cpu_bomb_clock -= dt;
        for f in &mut self.flames {
            f.life -= dt;
        }
        self.flames.retain(|f| f.life > 0.);
        for b in &mut self.bombs {
            b.fuse -= dt;
        }
        let mut queue: VecDeque<usize> = self
            .bombs
            .iter()
            .enumerate()
            .filter(|(_, b)| b.fuse <= 0.)
            .map(|(i, _)| i)
            .collect();
        let mut exploded = vec![false; self.bombs.len()];
        let mut hit = [false; N];
        while let Some(i) = queue.pop_front() {
            if exploded[i] {
                continue;
            }
            exploded[i] = true;
            for c in self.rays(self.bombs[i]) {
                hit[c] = true;
                for (j, b) in self.bombs.iter().enumerate() {
                    if b.cell == c && !exploded[j] {
                        queue.push_back(j);
                    }
                }
            }
        }
        for (c, &yes) in hit.iter().enumerate() {
            if !yes {
                continue;
            }
            if self.tiles[c] == Tile::Crate {
                self.tiles[c] = Tile::Floor;
                self.destroyed += 1;
                self.pickups[c] = c % 5 == 0;
            } else {
                self.pickups[c] = false;
            }
            self.flames.push(Flame {
                cell: c,
                life: 0.55,
            });
        }
        let mut i = 0;
        self.bombs.retain(|_| {
            let keep = !exploded[i];
            i += 1;
            keep
        });
        for who in 0..2 {
            if self.flames.iter().any(|f| f.cell == self.pawns[who]) {
                self.alive[who] = false;
            }
        }
        self.outcome = match self.alive {
            [false, false] => Some(End::Draw),
            [true, false] => Some(End::Win(0)),
            [false, true] => Some(End::Win(1)),
            _ if self.remaining <= 0. => Some(End::Draw),
            _ => None,
        };
        if self.outcome.is_none() && ai && self.cpu_clock >= 0.17 {
            self.cpu_clock = 0.;
            self.cpu();
        }
        // A CPU move can enter a lingering flame only if it made an unsafe choice.
        if self.flames.iter().any(|f| f.cell == self.pawns[1]) {
            self.alive[1] = false;
            self.outcome = Some(if self.alive[0] {
                End::Win(0)
            } else {
                End::Draw
            });
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn empty() -> Arena {
        let mut a = Arena::new();
        for t in &mut a.tiles {
            if *t == Tile::Crate {
                *t = Tile::Floor;
            }
        }
        a
    }
    #[test]
    fn wall_stops_and_crate_absorbs() {
        let mut a = empty();
        let b = Bomb {
            cell: W + 1,
            owner: 0,
            fuse: 1.,
            range: 5,
        };
        a.tiles[W + 3] = Tile::Crate;
        let rays = a.rays(b);
        assert!(rays.contains(&(W + 3)));
        assert!(!rays.contains(&(W + 4)));
        assert!(!rays.contains(&1));
    }
    #[test]
    fn chain_reaction_and_simultaneous_draw() {
        let mut a = empty();
        a.pawns = [W + 1, W + 3];
        a.bombs = vec![
            Bomb {
                cell: W + 1,
                owner: 0,
                fuse: 0.,
                range: 3,
            },
            Bomb {
                cell: W + 3,
                owner: 1,
                fuse: 2.,
                range: 3,
            },
        ];
        a.advance(0.01, false);
        assert!(a.bombs.is_empty());
        assert_eq!(a.outcome, Some(End::Draw));
    }
    #[test]
    fn capacity_and_no_reentry() {
        let mut a = empty();
        assert!(a.plant(0));
        assert!(!a.plant(0));
        assert!(a.walk(0, W + 2));
        assert!(!a.walk(0, W + 1));
        assert!(a.plant(0));
        assert!(a.walk(0, W + 3));
        assert!(!a.plant(0));
    }
    #[test]
    fn cpu_only_plants_with_escape() {
        let mut a = Arena::new();
        a.pawns[1] = W + 1;
        a.pawns[0] = W * 9 + 11;
        a.cpu();
        assert!(a.bombs.len() <= 1);
        if !a.bombs.is_empty() {
            assert!(a.escape(a.pawns[1], &a.bombs).is_some());
        }
    }
    #[test]
    fn cpu_match_is_bounded() {
        let mut a = Arena::new();
        for _ in 0..1300 {
            a.update(0.1);
            if a.outcome.is_some() {
                break;
            }
        }
        assert!(a.outcome.is_some());
    }
}
