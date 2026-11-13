//! Connect Four on Bloom: the alien cat is the only opponent.
#![allow(clippy::too_many_arguments)]
use crate::{
    characters::{Cast, Character},
    view::*,
};
use crownfall::{
    connect4::Connect4,
    reactions::{self, Mood},
};
use raylib::prelude::*;
#[derive(Clone, Copy)]
struct Fall {
    cell: usize,
    side: u8,
    age: f32,
}
fn draw(
    d: &mut impl RaylibDraw,
    f: &Fonts,
    p: &Pointer,
    cast: &Cast,
    g: &Connect4,
    fall: Option<Fall>,
    age: f32,
    time: f32,
    help: bool,
) -> Option<usize> {
    d.clear_background(Color::new(231, 247, 240, 255));
    text(d, f, "ORBIT CLUB / BLOOM", 32., 23., 15., BLUE, true);
    text(d, f, "CONNECT FOUR", 32., 52., 32., DARK, true);
    text(
        d,
        f,
        "You vs Mochi CPU  /  H returns to your ship",
        34.,
        96.,
        16.,
        GREY,
        false,
    );
    let moods = reactions::pulse(reactions::connect4(g), age, if g.turn == 1 { 1 } else { 0 });
    cast.duel(d, f, Character::Cat, moods, time);
    panel(d, rect(25., 143., 805., 661.));
    panel(d, rect(850., 143., 402., 661.));
    let (bx, by, s) = (99., 235., 91.);
    let hover = if p.point.x >= bx
        && p.point.x < bx + s * 7.
        && p.point.y >= 166.
        && p.point.y < by + s * 6.
    {
        Some(((p.point.x - bx) / s) as usize)
    } else {
        None
    };
    let ready = g.turn == 1 && g.winner.is_none() && !g.draw && fall.is_none() && !help;
    d.draw_rectangle_rounded(
        rect(bx - 17., by - 17., s * 7. + 34., s * 6. + 34.),
        0.07,
        16,
        Color::new(69, 158, 204, 255),
    );
    for col in 0..7 {
        let x = bx + col as f32 * s + s / 2.;
        text(d, f, &(col + 1).to_string(), x - 5., 185., 16., GREY, true);
        if hover == Some(col) && ready && g.landing(col).is_some() {
            d.draw_circle(x as i32, 213, 29., Color::new(253, 201, 91, 210));
        }
    }
    for cell in 0..42 {
        let x = bx + (cell % 7) as f32 * s + s / 2.;
        let y = by + (cell / 7) as f32 * s + s / 2.;
        d.draw_circle(
            x as i32,
            (y + 3.) as i32,
            38.,
            Color::new(47, 123, 166, 255),
        );
        d.draw_circle(x as i32, y as i32, 36., Color::new(230, 246, 241, 255));
        let value = g.board[cell];
        if value != 0 && fall.is_none_or(|a| a.cell != cell) {
            disc(d, x, y, value);
        }
        if g.line.contains(&cell) {
            d.draw_circle_lines(x as i32, y as i32, 41., Color::new(255, 235, 142, 255));
            d.draw_circle_lines(x as i32, y as i32, 43., Color::new(255, 235, 142, 255));
        }
    }
    if let Some(a) = fall {
        let t = (a.age / 0.45).clamp(0., 1.);
        let x = bx + (a.cell % 7) as f32 * s + s / 2.;
        let target = by + (a.cell / 7) as f32 * s + s / 2.;
        disc(d, x, by - 50. + (target - by + 50.) * t * t, a.side);
    }
    let status = match g.winner {
        Some(1) => "FOUR IN A ROW! YOU WIN",
        Some(_) => "MOCHI WINS",
        _ if g.draw => "A FULL-BOARD DRAW",
        _ if ready => "YOUR TURN",
        _ => "MOCHI IS THINKING...",
    };
    text(d, f, status, 873., 165., 21., BLUE, true);
    cast.body(
        d,
        Character::Cat,
        moods[1],
        rect(917., 226., 259., 259.),
        time,
    );
    centered(
        d,
        f,
        if moods[1] == Mood::Nervous {
            "Mochi sees your winning threat!"
        } else {
            moods[1].label()
        },
        1050.,
        491.,
        16.,
        DARK,
        true,
    );
    wrap(
        d,
        f,
        if help {
            "Drop one disc into a column. The first player with four in a horizontal, vertical or diagonal line wins. Full columns reject a drop. A filled board without a winning line is a draw."
        } else {
            "Click a column or press 1-7 to drop your gold disc. Connect four before Mochi does! The cat's portrait reacts to your moves and gets nervous when you have a winning threat."
        },
        875.,
        536.,
        348.,
        18.,
        DARK,
    );
    if button(
        d,
        f,
        p,
        rect(873., 669., 169., 42.),
        "Undo / Z",
        false,
        g.ply > 0,
    ) {
        return Some(10);
    }
    if button(
        d,
        f,
        p,
        rect(1054., 669., 174., 42.),
        "Rematch / R",
        false,
        true,
    ) {
        return Some(11);
    }
    if button(
        d,
        f,
        p,
        rect(873., 726., 169., 43.),
        if help { "Close rules" } else { "Rules / F1" },
        false,
        true,
    ) {
        return Some(12);
    }
    if button(
        d,
        f,
        p,
        rect(1054., 726., 174., 43.),
        "< Hub / H",
        true,
        true,
    ) {
        return Some(13);
    }
    if ready
        && p.clicked
        && let Some(c) = hover
    {
        return Some(c);
    }
    None
}
fn disc(d: &mut impl RaylibDraw, x: f32, y: f32, side: u8) {
    let c = if side == 1 {
        Color::new(255, 201, 84, 255)
    } else {
        Color::new(158, 203, 80, 255)
    };
    d.draw_circle(x as i32, (y + 2.) as i32, 35., Color::new(55, 96, 113, 75));
    d.draw_circle(x as i32, y as i32, 34., c);
    if side == 1 {
        d.draw_ellipse_lines(x as i32, y as i32, 44., 12., Color::new(190, 121, 52, 255));
    } else {
        for (dx, dy, rr) in [(-12., -8., 10.), (9., 12., 12.), (15., -14., 6.)] {
            d.draw_circle(
                (x + dx) as i32,
                (y + dy) as i32,
                rr,
                Color::new(56, 145, 184, 255),
            );
        }
    }
    d.draw_circle(
        (x - 9.) as i32,
        (y - 10.) as i32,
        9.,
        Color::new(255, 255, 255, 70),
    );
}
pub fn play(
    rl: &mut RaylibHandle,
    t: &RaylibThread,
    cast: &Cast,
) -> Result<Option<i8>, Box<dyn std::error::Error>> {
    rl.set_window_title(t, "Orbit Club / Connect Four vs Mochi CPU");
    let f = Fonts::load(rl, t);
    let mut g = Connect4::new();
    let mut history = vec![];
    let mut fall: Option<Fall> = None;
    let mut thinking = 0.;
    let mut time = 0.;
    let mut age = 10.;
    let mut help = false;
    let mut result = None;
    while !rl.window_should_close() {
        let dt = rl.get_frame_time().clamp(0., 0.1);
        time += dt;
        age += dt;
        let scale =
            (rl.get_screen_width() as f32 / WIDTH).min(rl.get_screen_height() as f32 / HEIGHT);
        let offset = Vector2::new(
            (rl.get_screen_width() as f32 - WIDTH * scale) * 0.5,
            (rl.get_screen_height() as f32 - HEIGHT * scale) * 0.5,
        );
        let m = rl.get_mouse_position();
        let p = Pointer {
            point: Vector2::new((m.x - offset.x) / scale, (m.y - offset.y) / scale),
            clicked: rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT),
        };
        if rl.is_key_pressed(KeyboardKey::KEY_H) {
            return Ok(result);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F1) {
            help = !help;
        }
        if !help {
            if let Some(a) = fall.as_mut() {
                a.age += dt;
                if a.age >= 0.45 {
                    fall = None;
                }
            } else if g.turn == 2 && g.winner.is_none() && !g.draw {
                thinking += dt;
                if thinking > 0.7 {
                    if let Some(c) = g.cpu()
                        && let Some(cell) = g.play(c)
                    {
                        fall = Some(Fall {
                            cell,
                            side: 2,
                            age: 0.,
                        });
                        age = 0.;
                    }
                    thinking = 0.;
                }
            }
        }
        if result != Some(1) {
            if let Some(w) = g.winner {
                result = Some(if w == 1 { 1 } else { -1 });
            } else if g.draw {
                result = Some(0);
            }
        }
        let mut action = {
            let mut d = rl.begin_drawing(t);
            d.clear_background(EDGE);
            let mut d = d.begin_mode2D(Camera2D {
                offset,
                target: Vector2::zero(),
                rotation: 0.,
                zoom: scale,
            });
            draw(&mut d, &f, &p, cast, &g, fall, age, time, help)
        };
        if rl.is_key_pressed(KeyboardKey::KEY_Z) {
            action = Some(10);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            action = Some(11);
        }
        if g.turn == 1 && fall.is_none() && !help {
            for (c, k) in [
                KeyboardKey::KEY_ONE,
                KeyboardKey::KEY_TWO,
                KeyboardKey::KEY_THREE,
                KeyboardKey::KEY_FOUR,
                KeyboardKey::KEY_FIVE,
                KeyboardKey::KEY_SIX,
                KeyboardKey::KEY_SEVEN,
            ]
            .iter()
            .enumerate()
            {
                if rl.is_key_pressed(*k) {
                    action = Some(c);
                }
            }
        }
        match action {
            Some(c @ 0..=6) if g.turn == 1 && fall.is_none() && !help => {
                let old = g.clone();
                if let Some(cell) = g.play(c) {
                    history.push(old);
                    fall = Some(Fall {
                        cell,
                        side: 1,
                        age: 0.,
                    });
                    thinking = 0.;
                    age = 0.;
                }
            }
            Some(10) => {
                if let Some(old) = history.pop() {
                    g = old;
                    fall = None;
                    thinking = 0.;
                    age = 10.;
                }
            }
            Some(11) => {
                g = Connect4::new();
                history.clear();
                fall = None;
                thinking = 0.;
                age = 10.;
            }
            Some(12) => help = !help,
            Some(13) => return Ok(result),
            _ => {}
        }
    }
    Ok(result)
}
pub fn capture(d: &mut impl RaylibDraw, f: &Fonts, p: &Pointer, cast: &Cast) {
    let mut g = Connect4::new();
    for c in [0, 6, 1, 6, 2, 5] {
        g.play(c);
    }
    draw(d, f, p, cast, &g, None, 1., 0., false);
}
