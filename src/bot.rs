//! Bot-only opponent: shortest-path racing, tactical jumps, and route-preserving barriers.
use crate::game::*;
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Level {
    Casual,
    Club,
    Pro,
}
impl Level {
    pub fn name(self) -> &'static str {
        match self {
            Self::Casual => "Casual",
            Self::Club => "Club",
            Self::Pro => "Pro",
        }
    }
}
pub fn choose(g: &Game, level: Level) -> Action {
    let me = g.turn;
    let them = 1 - me;
    let distances = g.distances(me);
    let mut moves = g.moves(me);
    moves.sort_by_key(|&c| {
        (
            if goal(me, c) { 0 } else { distances[c] + 1 },
            g.last[me] == Some(c),
            xy(c).0.abs_diff(4),
        )
    });
    let best_move = Action::Move(moves[0]);
    if goal(me, moves[0]) {
        return best_move;
    }
    if g.left[me] == 0 || level == Level::Casual && g.ply % 3 != 2 {
        return best_move;
    }
    let my_base = g.distance(me) as f32;
    let their_base = g.distance(them) as f32;
    let urgent = g.moves(them).iter().any(|&c| goal(them, c));
    let mut best_score = if urgent {
        -100.
    } else {
        match level {
            Level::Casual => 1.5,
            Level::Club => 0.4,
            Level::Pro => 0.05,
        }
    };
    let mut action = best_move;
    for y in 0..8 {
        for x in 0..8 {
            for axis in [Axis::Horizontal, Axis::Vertical] {
                if g.wall_valid(x, y, axis).is_err() {
                    continue;
                }
                let mut trial = g.clone();
                trial.walls.push(Wall {
                    x,
                    y,
                    axis,
                    owner: me,
                });
                let mine = trial.distance(me) as f32;
                let theirs = trial.distance(them) as f32;
                let still_wins = trial.moves(them).iter().any(|&c| goal(them, c));
                if urgent && still_wins {
                    continue;
                }
                let mut score = (theirs - their_base) - (mine - my_base) * 1.1 - 0.65;
                if g.left[me] < 3 {
                    score -= 0.35;
                }
                if their_base <= my_base && their_base < 5. {
                    score += 0.25;
                }
                if urgent {
                    score += 20.;
                }
                if level == Level::Pro {
                    let response = trial
                        .moves(them)
                        .into_iter()
                        .map(|c| trial.distances(them)[c])
                        .min()
                        .unwrap_or(99) as f32;
                    score += (response - theirs) * 0.25;
                }
                if score > best_score + 0.0001 {
                    best_score = score;
                    action = Action::Place { x, y, axis };
                }
            }
        }
    }
    action
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bot_finishes_immediate_goal() {
        let mut g = Game::new();
        g.turn = 1;
        g.pawns = [45, 67];
        assert_eq!(choose(&g, Level::Club), Action::Move(76));
    }
    #[test]
    fn games_finish_and_all_actions_are_legal() {
        for seed in 0..16 {
            let mut g = Game::new();
            for ply in 0..240 {
                if g.winner.is_some() {
                    break;
                }
                let level = if (seed + ply) % 4 == 0 {
                    Level::Casual
                } else {
                    Level::Club
                };
                let a = choose(&g, level);
                assert!(g.validate(a).is_ok(), "{a:?}");
                g.apply(a).unwrap();
                assert!(g.distance(0) < u16::MAX && g.distance(1) < u16::MAX);
                assert_ne!(g.pawns[0], g.pawns[1]);
            }
            assert!(g.winner.is_some(), "match {seed} stalled");
        }
    }
}
