//! Character moods are driven by the real rules state, never a random expression timer.
use crate::{
    bomber::{Arena, End},
    chess::{Chess, Outcome},
    connect4::Connect4,
    game::Game,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Mood {
    Idle,
    Thinking,
    Nervous,
    Happy,
    Sad,
    Surprised,
}
impl Mood {
    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "Ready!",
            Self::Thinking => "Thinking...",
            Self::Nervous => "Uh oh...",
            Self::Happy => "Hooray!",
            Self::Sad => "Good game",
            Self::Surprised => "Oh! Nice move",
        }
    }
}
pub fn duel(human_danger: bool, cpu_danger: bool, cpu_turn: bool, result: Option<i8>) -> [Mood; 2] {
    match result {
        Some(1) => [Mood::Happy, Mood::Sad],
        Some(-1) => [Mood::Sad, Mood::Happy],
        Some(_) => [Mood::Surprised, Mood::Surprised],
        None => [
            if human_danger {
                Mood::Nervous
            } else {
                Mood::Idle
            },
            if cpu_danger {
                Mood::Nervous
            } else if cpu_turn {
                Mood::Thinking
            } else {
                Mood::Idle
            },
        ],
    }
}
pub fn quoridor(g: &Game) -> [Mood; 2] {
    duel(
        g.distance(1) <= 2,
        g.distance(0) <= 2,
        g.turn == 1,
        g.winner.map(|w| if w == 0 { 1 } else { -1 }),
    )
}
pub fn connect4(g: &Connect4) -> [Mood; 2] {
    duel(
        !g.winning_columns(2).is_empty(),
        !g.winning_columns(1).is_empty(),
        g.turn == 2,
        g.winner
            .map(|w| if w == 1 { 1 } else { -1 })
            .or(if g.draw { Some(0) } else { None }),
    )
}
fn mate_next(g: &Chess, side: i8) -> bool {
    let mut trial = g.clone();
    trial.turn = side;
    trial.ep = if g.turn == side { g.ep } else { None };
    trial.legal().into_iter().any(|m| {
        let mut next = trial.clone();
        next.play(m);
        next.outcome == Some(Outcome::Won(side))
    })
}
pub fn chess(g: &Chess) -> [Mood; 2] {
    let result = g.outcome.map(|o| match o {
        Outcome::Won(s) => s,
        Outcome::Draw(_) => 0,
    });
    let (human, cpu) = if result.is_none() {
        (
            g.in_check(1) || mate_next(g, -1),
            g.in_check(-1) || mate_next(g, 1),
        )
    } else {
        (false, false)
    };
    duel(human, cpu, g.turn == -1, result)
}
pub fn bomber(a: &Arena) -> [Mood; 2] {
    let threatened = |who: usize| {
        a.bombs
            .iter()
            .any(|&b| b.owner != who && a.rays(b).contains(&a.pawns[who]))
    };
    duel(
        threatened(0),
        threatened(1),
        !a.bombs.is_empty(),
        a.outcome.map(|o| match o {
            End::Win(0) => 1,
            End::Win(_) => -1,
            End::Draw => 0,
        }),
    )
}
/// A short surprise pulse after the rival acts; danger and results always take precedence.
pub fn pulse(mut moods: [Mood; 2], age: f32, last_actor: usize) -> [Mood; 2] {
    if age < 0.65 && matches!(moods[1 - last_actor], Mood::Idle | Mood::Thinking) {
        moods[1 - last_actor] = Mood::Surprised;
    }
    moods
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pig_gets_nervous_near_human_finish() {
        let mut g = Game::new();
        g.pawns[0] = 18;
        assert_eq!(quoridor(&g)[1], Mood::Nervous);
        g.winner = Some(0);
        assert_eq!(quoridor(&g), [Mood::Happy, Mood::Sad]);
    }
    #[test]
    fn cat_reacts_to_three_and_terminal_win() {
        let mut g = Connect4::new();
        for c in [0, 6, 1, 6, 2, 5] {
            g.play(c);
        }
        assert_eq!(connect4(&g)[1], Mood::Nervous);
        g.play(3);
        assert_eq!(connect4(&g), [Mood::Happy, Mood::Sad]);
    }
    #[test]
    fn chess_host_gets_nervous_about_mate_in_one() {
        let mut g = Chess::new();
        for (from, to) in [(52, 36), (12, 28), (61, 34), (1, 18), (59, 31), (6, 21)] {
            assert!(g.play(crate::chess::Move {
                from,
                to,
                promotion: 0
            }));
        }
        assert_eq!(chess(&g)[1], Mood::Nervous);
    }
    #[test]
    fn ember_reacts_to_a_real_incoming_blast() {
        let mut a = Arena::new();
        a.pawns[1] = crate::bomber::W + 3;
        a.tiles[crate::bomber::W + 2] = crate::bomber::Tile::Floor;
        a.tiles[crate::bomber::W + 3] = crate::bomber::Tile::Floor;
        assert!(a.plant(0));
        assert_eq!(bomber(&a)[1], Mood::Nervous);
    }
    #[test]
    fn draw_and_surprise_have_correct_priority() {
        assert_eq!(duel(false, false, false, Some(0)), [Mood::Surprised; 2]);
        assert_eq!(pulse([Mood::Idle, Mood::Nervous], 0.1, 0)[1], Mood::Nervous);
        assert_eq!(pulse([Mood::Idle; 2], 0.1, 0)[1], Mood::Surprised);
    }
}
