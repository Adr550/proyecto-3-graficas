//! Bright, rounded, readable console-sports interface; no Nintendo assets or logos.
#![allow(clippy::too_many_arguments)]
use crate::characters::{Cast, Character};
use crate::client::{Client, Command, Modal, Tool};
use crownfall::reactions;
use crownfall::{
    bot::Level,
    game::*,
    math::V3,
    scene::{Scene, biome, cell_position},
};
use raylib::prelude::*;
pub const WIDTH: f32 = 1280.;
pub const HEIGHT: f32 = 850.;
pub const SCENE: Rectangle = Rectangle {
    x: 24.,
    y: 164.,
    width: 910.,
    height: 625.,
};
pub(crate) const BLUE: Color = Color::new(38, 154, 219, 255);
pub(crate) const DARK: Color = Color::new(45, 93, 131, 255);
pub(crate) const GREY: Color = Color::new(114, 140, 159, 255);
pub(crate) const CORAL: Color = Color::new(238, 132, 118, 255);
pub(crate) const EDGE: Color = Color::new(194, 220, 236, 255);
pub(crate) const WHITE: Color = Color::new(255, 255, 255, 255);
pub struct Fonts {
    body: Option<Font>,
    rounded: Option<Font>,
}
impl Fonts {
    pub fn load(rl: &mut RaylibHandle, t: &RaylibThread) -> Self {
        let load = |rl: &mut RaylibHandle, paths: &[&str], size| {
            paths.iter().find_map(|path| {
                if std::path::Path::new(path).exists() {
                    rl.load_font_ex(t, path, size, None).ok().inspect(|f| {
                        f.texture()
                            .set_texture_filter(t, TextureFilter::TEXTURE_FILTER_BILINEAR)
                    })
                } else {
                    None
                }
            })
        };
        Self {
            body: load(
                rl,
                &[
                    "/System/Library/Fonts/Supplemental/Arial.ttf",
                    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                    "C:/Windows/Fonts/arial.ttf",
                ],
                40,
            ),
            rounded: load(
                rl,
                &[
                    "/System/Library/Fonts/Supplemental/Arial Rounded Bold.ttf",
                    "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
                    "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
                    "C:/Windows/Fonts/arialbd.ttf",
                ],
                60,
            ),
        }
    }
}
pub struct Pointer {
    pub point: Vector2,
    pub clicked: bool,
}
pub(crate) fn rect(x: f32, y: f32, w: f32, h: f32) -> Rectangle {
    Rectangle::new(x, y, w, h)
}
pub(crate) fn text(
    d: &mut impl RaylibDraw,
    f: &Fonts,
    s: &str,
    x: f32,
    y: f32,
    size: f32,
    color: Color,
    bold: bool,
) {
    if let Some(font) = if bold {
        f.rounded.as_ref()
    } else {
        f.body.as_ref()
    } {
        d.draw_text_ex(font, s, Vector2::new(x, y), size, 0.1, color);
    } else {
        d.draw_text(s, x as i32, y as i32, size as i32, color);
    }
}
pub(crate) fn centered(
    d: &mut impl RaylibDraw,
    f: &Fonts,
    s: &str,
    x: f32,
    y: f32,
    size: f32,
    color: Color,
    bold: bool,
) {
    let width = if let Some(font) = if bold {
        f.rounded.as_ref()
    } else {
        f.body.as_ref()
    } {
        font.measure_text(s, size, 0.1).x
    } else {
        s.len() as f32 * size * 0.5
    };
    text(d, f, s, x - width * 0.5, y, size, color, bold);
}
pub(crate) fn wrap(
    d: &mut impl RaylibDraw,
    f: &Fonts,
    s: &str,
    x: f32,
    mut y: f32,
    width: f32,
    size: f32,
    color: Color,
) {
    let mut line = String::new();
    for word in s.split_whitespace() {
        let next = if line.is_empty() {
            word.into()
        } else {
            format!("{line} {word}")
        };
        let measure = f
            .body
            .as_ref()
            .map_or(next.len() as f32 * size * 0.5, |font| {
                font.measure_text(&next, size, 0.1).x
            });
        if measure > width && !line.is_empty() {
            text(d, f, &line, x, y, size, color, false);
            y += size * 1.45;
            line = word.into();
        } else {
            line = next;
        }
    }
    if !line.is_empty() {
        text(d, f, &line, x, y, size, color, false);
    }
}
pub(crate) fn panel(d: &mut impl RaylibDraw, r: Rectangle) {
    d.draw_rectangle_rounded(
        rect(r.x, r.y + 4., r.width, r.height),
        0.12,
        14,
        Color::new(85, 142, 177, 26),
    );
    d.draw_rectangle_rounded(r, 0.12, 14, WHITE);
    d.draw_rectangle_rounded_lines_ex(r, 0.12, 14, 1.5, EDGE);
}
pub(crate) fn button(
    d: &mut impl RaylibDraw,
    f: &Fonts,
    p: &Pointer,
    r: Rectangle,
    label: &str,
    active: bool,
    enabled: bool,
) -> bool {
    let hover = enabled && r.check_collision_point_rec(p.point);
    let fill = if !enabled {
        Color::new(237, 244, 249, 255)
    } else if active {
        if hover {
            Color::new(74, 179, 235, 255)
        } else {
            BLUE
        }
    } else if hover {
        Color::new(226, 244, 255, 255)
    } else {
        WHITE
    };
    d.draw_rectangle_rounded(
        rect(r.x, r.y + 3., r.width, r.height),
        0.45,
        14,
        Color::new(71, 129, 165, 25),
    );
    d.draw_rectangle_rounded(r, 0.45, 14, fill);
    d.draw_rectangle_rounded_lines_ex(
        r,
        0.45,
        14,
        if active { 2. } else { 1.5 },
        if active {
            Color::new(23, 142, 205, 255)
        } else {
            EDGE
        },
    );
    if active {
        d.draw_rectangle_rounded(
            rect(r.x + 6., r.y + 4., r.width - 12., r.height * 0.29),
            0.6,
            10,
            Color::new(255, 255, 255, 60),
        );
    }
    centered(
        d,
        f,
        label,
        r.x + r.width * 0.5,
        r.y + (r.height - 16.) * 0.5,
        16.,
        if !enabled {
            Color::new(166, 188, 203, 255)
        } else if active {
            WHITE
        } else {
            DARK
        },
        true,
    );
    hover && p.clicked
}
fn score(d: &mut impl RaylibDraw, f: &Fonts, x: f32, player: usize, c: &Client, cast: &Cast) {
    let color = if player == 0 { BLUE } else { CORAL };
    panel(d, rect(x, 25., 190., 66.));
    let mood = reactions::pulse(reactions::quoridor(&c.game), c.reaction_age, c.last_actor)[player];
    cast.portrait(
        d,
        if player == 0 {
            Character::Fer
        } else {
            Character::Pig
        },
        mood,
        Vector2::new(x + 30., 56.),
        25.,
        c.clock,
    );
    text(
        d,
        f,
        if player == 0 {
            "FER / YOU"
        } else {
            "TRUFFLE / CPU"
        },
        x + 64.,
        35.,
        14.,
        color,
        true,
    );
    text(
        d,
        f,
        &format!("{} barriers", c.game.left[player]),
        x + 64.,
        58.,
        12.,
        GREY,
        false,
    );
    for i in 0..10 {
        d.draw_rectangle(
            (x + 64. + i as f32 * 9.) as i32,
            78,
            7,
            4,
            if i < c.game.left[player] as usize {
                color
            } else {
                Color::new(226, 235, 242, 255)
            },
        );
    }
}
pub fn draw(
    d: &mut impl RaylibDraw,
    f: &Fonts,
    c: &Client,
    s: &Scene,
    texture: &Texture2D,
    p: &Pointer,
    cast: &Cast,
) -> Option<Command> {
    let mut command = None;
    d.draw_rectangle_gradient_v(
        0,
        0,
        1280,
        850,
        Color::new(237, 249, 255, 255),
        Color::new(215, 237, 251, 255),
    );
    for i in 0..14 {
        d.draw_circle(
            60 + i * 93,
            120 + ((i * 131) % 650),
            25. + (i % 3) as f32 * 16.,
            Color::new(255, 255, 255, 45),
        );
    }
    // Original, friendly club branding rather than a Nintendo/Wii logo.
    d.draw_circle(47, 51, 23., BLUE);
    d.draw_ellipse_lines(47, 51, 31., 10., WHITE);
    d.draw_circle(54, 43, 8., Color::new(174, 228, 255, 255));
    text(d, f, "ORBIT PATH", 87., 31., 30., BLUE, true);
    text(
        d,
        f,
        "A Quoridor-inspired space duel",
        89.,
        67.,
        12.,
        GREY,
        false,
    );
    score(d, f, 399., 0, c, cast);
    score(d, f, 605., 1, c, cast);
    if button(
        d,
        f,
        p,
        rect(823., 35., 165., 43.),
        "How to play",
        false,
        true,
    ) {
        command = Some(Command::Rules);
    }
    if button(
        d,
        f,
        p,
        rect(1006., 35., 248., 43.),
        "New match  +",
        false,
        true,
    ) {
        command = Some(Command::NewMatch);
    }
    d.draw_line_ex(
        Vector2::new(25., 111.),
        Vector2::new(1255., 111.),
        1.,
        Color::new(192, 222, 242, 255),
    );
    text(d, f, "PLANET COURT  /  9 x 9", 35., 132., 12., GREY, true);
    centered(
        d,
        f,
        if c.game.winner.is_some() {
            "MATCH COMPLETE"
        } else if c.game.turn == 0 {
            "YOUR TURN!"
        } else {
            "TRUFFLE IS THINKING..."
        },
        477.,
        126.,
        21.,
        if c.game.turn == 0 { BLUE } else { CORAL },
        true,
    );
    text(
        d,
        f,
        &format!("ROUND {:02}  /  {}", c.game.ply / 2 + 1, c.level.name()),
        735.,
        133.,
        12.,
        GREY,
        false,
    );
    d.draw_rectangle_rounded(rect(20., 160., 918., 633.), 0.04, 10, WHITE);
    d.draw_texture_pro(
        texture,
        rect(0., 0., s.renderer.width as f32, s.renderer.height as f32),
        SCENE,
        Vector2::zero(),
        0.,
        WHITE,
    );
    d.draw_rectangle_rounded_lines_ex(rect(21., 161., 916., 631.), 0.035, 12, 2., WHITE);
    // Finish-line labels live in screen space, while the row rings are true 3D geometry.
    for (row, label, color) in [(0, "YOUR FINISH", BLUE), (8, "TRUFFLE'S FINISH", CORAL)] {
        let point = s.renderer.project(
            cell_position(row * 9 + 4) + V3::new(0., -1., if row == 0 { -10. } else { 10. }),
        );
        let x = SCENE.x + point.x;
        let y = SCENE.y + point.y;
        d.draw_rectangle_rounded(
            rect(x - 71., y - 9., 142., 23.),
            0.8,
            10,
            Color::new(255, 255, 255, 220),
        );
        centered(d, f, label, x, y - 5., 11., color, true);
    }
    for player in 0..2 {
        let q = s
            .renderer
            .project(s.ship_positions[player] + V3::new(0., 7., 0.));
        if q.w > 0. {
            let x = SCENE.x + q.x;
            let y = SCENE.y + q.y;
            let color = if player == 0 { BLUE } else { CORAL };
            d.draw_rectangle_rounded(rect(x - 29., y - 14., 58., 23.), 0.8, 10, color);
            centered(
                d,
                f,
                if player == 0 { "YOU" } else { "TRUFFLE" },
                x,
                y - 10.,
                11.,
                WHITE,
                true,
            );
        }
    }
    if c.human_ready()
        && c.tool == Tool::Move
        && let Some(cell) = c.hover_cell
    {
        let q = s
            .renderer
            .project(cell_position(cell) + V3::new(0., 5., 0.));
        let x = SCENE.x + q.x;
        let y = SCENE.y + q.y;
        if c.game.moves(0).contains(&cell) {
            d.draw_circle_lines(x as i32, y as i32, 17., BLUE);
            d.draw_rectangle_rounded(rect(x - 43., y + 20., 86., 24.), 0.8, 8, WHITE);
            centered(
                d,
                f,
                &format!("Fly to {}", coord(cell)),
                x,
                y + 24.,
                11.,
                BLUE,
                true,
            );
        }
    }
    panel(d, rect(958., 164., 298., 625.));
    let x = 981.;
    let ready = c.human_ready();
    text(
        d,
        f,
        if c.game.winner.is_some() {
            "Nice match!"
        } else if c.game.turn == 0 {
            "Let's go!"
        } else {
            "Your rival's turn"
        },
        x,
        187.,
        25.,
        if c.game.turn == 0 { BLUE } else { CORAL },
        true,
    );
    wrap(
        d,
        f,
        if c.game.turn == 0 {
            "Move your ship OR place one barrier. Reach the far blue edge to win."
        } else {
            "Truffle uses the same rules and barriers as you. Watch the next move!"
        },
        x,
        229.,
        250.,
        14.,
        GREY,
    );
    if button(
        d,
        f,
        p,
        rect(x, 295., 252., 54.),
        "Move ship   [M]",
        c.tool == Tool::Move,
        ready,
    ) {
        command = Some(Command::MoveTool);
    }
    if button(
        d,
        f,
        p,
        rect(x, 365., 252., 54.),
        "Build barrier   [B]",
        c.tool == Tool::Barrier,
        ready && c.game.left[0] > 0,
    ) {
        command = Some(Command::BarrierTool);
    }
    text(d, f, "BARRIER DIRECTION", x, 443., 11., GREY, true);
    if button(
        d,
        f,
        p,
        rect(x, 466., 252., 37.),
        if c.axis == Axis::Horizontal {
            "Across  <---->   [R]"
        } else {
            "Along  /   [R]"
        },
        false,
        c.tool == Tool::Barrier && ready,
    ) {
        command = Some(Command::Rotate);
    }
    let message = if c.tool == Tool::Barrier {
        if let Some((wx, wy)) = c.hover_wall {
            match c.game.validate(Action::Place {
                x: wx,
                y: wy,
                axis: c.axis,
            }) {
                Ok(()) => "Looks good! Click to build this barrier.",
                Err(message) => message,
            }
        } else {
            "Hover a gap between four planets. Press R to rotate the barrier."
        }
    } else {
        "Click a glowing neighboring planet. Jump over Truffle when the route is clear."
    };
    let okay = c.tool != Tool::Barrier
        || c.hover_wall.is_none_or(|(wx, wy)| {
            c.game
                .validate(Action::Place {
                    x: wx,
                    y: wy,
                    axis: c.axis,
                })
                .is_ok()
        });
    d.draw_rectangle_rounded(
        rect(x, 523., 252., 75.),
        0.15,
        10,
        if okay {
            Color::new(236, 248, 255, 255)
        } else {
            Color::new(255, 238, 237, 255)
        },
    );
    wrap(
        d,
        f,
        message,
        x + 12.,
        536.,
        228.,
        13.,
        if okay { DARK } else { CORAL },
    );
    if button(
        d,
        f,
        p,
        rect(x, 618., 252., 41.),
        "Undo round   [Z]",
        false,
        !c.history.is_empty(),
    ) {
        command = Some(Command::Undo);
    }
    if button(
        d,
        f,
        p,
        rect(x, 678., 119., 34.),
        if c.hints { "Hints: ON" } else { "Hints: OFF" },
        false,
        true,
    ) {
        command = Some(Command::ToggleHints);
    }
    if button(
        d,
        f,
        p,
        rect(x + 132., 678., 120., 34.),
        "Reset view",
        false,
        true,
    ) {
        command = Some(Command::ResetView);
    }
    if button(
        d,
        f,
        p,
        rect(x, 730., 252., 36.),
        "View from above",
        false,
        true,
    ) {
        command = Some(Command::TopView);
    }
    cast.body(
        d,
        Character::Pig,
        reactions::pulse(reactions::quoridor(&c.game), c.reaction_age, c.last_actor)[1],
        rect(35., 178., 98., 98.),
        c.clock,
    );
    let description = if c.notice_time > 0. {
        c.notice.as_str()
    } else {
        c.game
            .log
            .front()
            .map(String::as_str)
            .unwrap_or("Race to the opposite edge!")
    };
    text(d, f, description, 214., 805., 14., DARK, false);
    text(
        d,
        f,
        "Right-drag: orbit   /   Scroll: zoom   /   V: reset",
        214.,
        832.,
        10.,
        GREY,
        false,
    );
    text(d, f, "YOUR SHIP: ", 552., 830., 10., GREY, true);
    text(
        d,
        f,
        &format!(
            "{} / {}",
            coord(c.game.pawns[0]),
            s.world.planets[biome(c.game.pawns[0])].def.name
        ),
        626.,
        830.,
        10.,
        GREY,
        false,
    );
    if button(
        d,
        f,
        p,
        rect(1006., 802., 248., 33.),
        "Save view",
        false,
        true,
    ) {
        command = Some(Command::Snapshot);
    }
    if c.modal != Modal::None || c.game.winner.is_some() {
        d.draw_rectangle(0, 0, 1280, 850, Color::new(99, 156, 192, 154));
        panel(d, rect(315., 112., 650., 625.));
        let x = 353.;
        if c.game.winner.is_some() && c.modal == Modal::None {
            let won = c.game.winner == Some(0);
            d.draw_circle(640, 211, 48., if won { BLUE } else { CORAL });
            centered(
                d,
                f,
                if won { "1st" } else { "2nd" },
                640.,
                185.,
                38.,
                WHITE,
                true,
            );
            centered(
                d,
                f,
                if won { "Great flying!" } else { "Good game!" },
                640.,
                286.,
                38.,
                BLUE,
                true,
            );
            centered(
                d,
                f,
                if won {
                    "You crossed the planet court first."
                } else {
                    "Truffle reached the finish this time."
                },
                640.,
                346.,
                19.,
                DARK,
                false,
            );
            centered(
                d,
                f,
                &format!(
                    "{} turns  /  {} barriers used",
                    c.game.ply,
                    20 - c.game.left[0] - c.game.left[1]
                ),
                640.,
                398.,
                15.,
                GREY,
                false,
            );
            if button(
                d,
                f,
                p,
                rect(x, 487., 574., 58.),
                "Play again against Truffle",
                true,
                true,
            ) {
                command = Some(Command::Start(c.level));
            }
            if button(
                d,
                f,
                p,
                rect(x, 567., 574., 45.),
                "Change difficulty",
                false,
                true,
            ) {
                command = Some(Command::NewMatch);
            }
            centered(
                d,
                f,
                "One player. One computer rival. One more race?",
                640.,
                661.,
                13.,
                GREY,
                false,
            );
        } else if c.modal == Modal::Rules {
            text(d, f, "HOW TO PLAY", x, 151., 12., BLUE, true);
            text(
                d,
                f,
                "A simple race. A clever route.",
                x,
                183.,
                30.,
                DARK,
                true,
            );
            let rows = [
                (
                    "1",
                    "Move OR build",
                    "On your turn, fly one planet horizontally or vertically, or place one of your ten barriers.",
                ),
                (
                    "2",
                    "Keep a route open",
                    "Each barrier spans two neighboring lanes. It cannot overlap, cross another barrier, or seal either ship away from its finish.",
                ),
                (
                    "3",
                    "Jump past your rival",
                    "If Truffle is next to you, jump straight over it. If that jump is blocked by a barrier or board edge, hop to an open side instead.",
                ),
                (
                    "4",
                    "Reach the far edge",
                    "You start at E9 and race toward row 1. Truffle starts at E1 and races toward row 9. First across wins!",
                ),
            ];
            for (i, (n, title, body)) in rows.iter().enumerate() {
                let y = 253. + i as f32 * 88.;
                d.draw_circle((x + 16.) as i32, (y + 15.) as i32, 16., BLUE);
                centered(d, f, n, x + 16., y + 4., 18., WHITE, true);
                text(d, f, title, x + 48., y, 17., DARK, true);
                wrap(d, f, body, x + 48., y + 28., 516., 13., GREY);
            }
            if button(
                d,
                f,
                p,
                rect(x, 649., 574., 47.),
                "Got it! Back to the match",
                true,
                true,
            ) {
                command = Some(Command::Close);
            }
        } else {
            text(d, f, "ONE PLAYER / VS TRUFFLE", x, 155., 12., BLUE, true);
            text(d, f, "Ready for a new race?", x, 192., 34., DARK, true);
            wrap(
                d,
                f,
                "Choose your computer rival's level. A new match replaces the current race.",
                x,
                253.,
                565.,
                17.,
                GREY,
            );
            let levels = [
                (
                    Level::Casual,
                    "Casual",
                    "A relaxed rival that mostly races forward.",
                ),
                (
                    Level::Club,
                    "Club",
                    "Plans routes and places tactical barriers.",
                ),
                (
                    Level::Pro,
                    "Pro",
                    "Uses stronger blocking and response analysis.",
                ),
            ];
            for (i, (level, name, description)) in levels.iter().enumerate() {
                let y = 342. + i as f32 * 89.;
                if button(
                    d,
                    f,
                    p,
                    rect(x, y, 574., 48.),
                    name,
                    c.level == *level,
                    true,
                ) {
                    command = Some(Command::Start(*level));
                }
                text(d, f, description, x + 16., y + 57., 12., GREY, false);
            }
            if button(
                d,
                f,
                p,
                rect(x, 649., 574., 45.),
                "Keep playing",
                false,
                true,
            ) {
                command = Some(Command::Close);
            }
        }
    }
    command
}
