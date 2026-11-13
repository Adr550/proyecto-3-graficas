use crownfall::{
    bot::{self, Level},
    game::*,
    math::V3,
    scene::cell_position,
};
#[derive(Clone, Copy, PartialEq)]
pub enum Tool {
    Move,
    Barrier,
}
#[derive(Clone, Copy, PartialEq)]
pub enum Modal {
    None,
    Rules,
    NewMatch,
}
#[derive(Clone, Copy)]
pub enum Command {
    MoveTool,
    BarrierTool,
    Rotate,
    Undo,
    Rules,
    Close,
    NewMatch,
    Start(Level),
    ResetView,
    TopView,
    Snapshot,
    ToggleHints,
}
pub struct Motion {
    pub player: usize,
    pub from: V3,
    pub to: V3,
    pub progress: f32,
}
pub struct Client {
    pub game: Game,
    pub level: Level,
    pub tool: Tool,
    pub axis: Axis,
    pub modal: Modal,
    pub hover_cell: Option<usize>,
    pub hover_wall: Option<(u8, u8)>,
    pub notice: String,
    pub notice_time: f32,
    pub clock: f32,
    pub reaction_age: f32,
    pub last_actor: usize,
    pub thinking: f32,
    pub motion: Option<Motion>,
    pub history: Vec<Game>,
    pub hints: bool,
    pub matches: u32,
    pub wins: u32,
    pub scored: bool,
}
impl Client {
    pub fn new(level: Level) -> Self {
        Self {
            game: Game::new(),
            level,
            tool: Tool::Move,
            axis: Axis::Horizontal,
            modal: Modal::None,
            hover_cell: None,
            hover_wall: None,
            notice: "Click a glowing planet to move. Or build a barrier!".into(),
            notice_time: 6.,
            clock: 0.,
            reaction_age: 10.,
            last_actor: 1,
            thinking: 0.,
            motion: None,
            history: vec![],
            hints: true,
            matches: 0,
            wins: 0,
            scored: false,
        }
    }
    pub fn human_ready(&self) -> bool {
        self.game.turn == 0
            && self.game.winner.is_none()
            && self.modal == Modal::None
            && self.motion.is_none()
    }
    pub fn notify(&mut self, text: impl Into<String>) {
        self.notice = text.into();
        self.notice_time = 4.;
    }
    fn animate(&mut self, old: &Game) {
        let player = old.turn;
        if old.pawns[player] != self.game.pawns[player] {
            self.motion = Some(Motion {
                player,
                from: cell_position(old.pawns[player]) + V3::new(0., 7.2, 0.),
                to: cell_position(self.game.pawns[player]) + V3::new(0., 7.2, 0.),
                progress: 0.,
            });
        }
    }
    pub fn human_action(&mut self, a: Action) {
        if !self.human_ready() {
            return;
        }
        if let Err(message) = self.game.validate(a) {
            self.notify(message);
            return;
        }
        let old = self.game.clone();
        self.history.push(old.clone());
        self.game.apply(a).unwrap();
        self.animate(&old);
        self.reaction_age = 0.;
        self.last_actor = 0;
        self.thinking = 0.;
        self.tool = Tool::Move;
        self.hover_wall = None;
        self.notify(if self.game.winner == Some(0) {
            "Nice finish! You crossed the planet board!"
        } else {
            "Truffle is planning a move..."
        });
    }
    pub fn update(&mut self, dt: f32) {
        self.clock += dt;
        self.reaction_age += dt;
        self.notice_time = (self.notice_time - dt).max(0.);
        if self.modal != Modal::None {
            return;
        }
        if let Some(m) = self.motion.as_mut() {
            m.progress += dt / 0.55;
            if m.progress >= 1. {
                self.motion = None;
            } else {
                return;
            }
        }
        if self.game.winner.is_some() {
            if !self.scored {
                self.scored = true;
                self.matches += 1;
                if self.game.winner == Some(0) {
                    self.wins += 1;
                }
            }
            return;
        }
        if self.game.turn == 1 {
            self.thinking += dt;
            if self.thinking > 0.72 {
                let action = bot::choose(&self.game, self.level);
                let old = self.game.clone();
                self.game
                    .apply(action)
                    .expect("Truffle must obey the same rules");
                self.animate(&old);
                self.reaction_age = 0.;
                self.last_actor = 1;
                self.thinking = 0.;
                self.notify(if self.game.winner == Some(1) {
                    "Truffle crossed the finish. Let's try again!"
                } else {
                    "Your turn! Move your ship or place one barrier."
                });
            }
        }
    }
    pub fn ship_positions(&self) -> [V3; 2] {
        let mut p = self
            .game
            .pawns
            .map(|cell| cell_position(cell) + V3::new(0., 7.2, 0.));
        if let Some(m) = &self.motion {
            let t = m.progress.clamp(0., 1.);
            let e = t * t * (3. - 2. * t);
            p[m.player] =
                m.from.lerp(m.to, e) + V3::new(0., (t * std::f32::consts::PI).sin() * 3., 0.);
        }
        p
    }
    pub fn command(&mut self, c: Command) {
        match c {
            Command::MoveTool => self.tool = Tool::Move,
            Command::BarrierTool => {
                if self.game.left[0] > 0 {
                    self.tool = Tool::Barrier;
                    self.notify("Hover a gap, then click to build. R rotates the preview.");
                } else {
                    self.notify("No barriers left. Fly your ship to the finish!");
                }
            }
            Command::Rotate => {
                self.axis = self.axis.other();
            }
            Command::Undo => {
                if let Some(old) = self.history.pop() {
                    self.game = old;
                    self.motion = None;
                    self.thinking = 0.;
                    self.scored = false;
                    self.tool = Tool::Move;
                    self.notify("Round undone. Try a new route!");
                }
            }
            Command::Rules => self.modal = Modal::Rules,
            Command::Close => self.modal = Modal::None,
            Command::NewMatch => self.modal = Modal::NewMatch,
            Command::Start(level) => {
                let (wins, matches) = (self.wins, self.matches);
                *self = Self::new(level);
                self.wins = wins;
                self.matches = matches;
            }
            Command::ToggleHints => self.hints = !self.hints,
            Command::ResetView | Command::TopView | Command::Snapshot => {}
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_the_human_ship_can_be_controlled() {
        let mut c = Client::new(Level::Club);
        c.human_action(Action::Move(67));
        let old = c.game.pawns;
        c.human_action(Action::Move(13));
        assert_eq!(c.game.pawns, old);
        assert_eq!(c.game.turn, 1);
    }
    #[test]
    fn undo_restores_a_full_round() {
        let mut c = Client::new(Level::Club);
        c.human_action(Action::Move(67));
        for _ in 0..100 {
            c.update(0.02);
        }
        assert_eq!(c.game.turn, 0);
        c.command(Command::Undo);
        assert_eq!(c.game.pawns, [76, 4]);
        assert_eq!(c.game.left, [10, 10]);
        assert_eq!(c.game.turn, 0);
    }
    #[test]
    fn help_pauses_the_bot() {
        let mut c = Client::new(Level::Club);
        c.human_action(Action::Move(67));
        c.motion = None;
        c.command(Command::Rules);
        c.update(2.);
        assert_eq!(c.game.pawns[1], 4);
        c.command(Command::Close);
        c.update(1.);
        assert_eq!(c.game.turn, 0);
    }
}
