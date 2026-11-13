//! Mouse-driven chess and real-time bomb arena screens, always against a CPU.
#![allow(clippy::too_many_arguments)]
use crate::characters::{Cast, Character};
use crate::view::*;
use crownfall::reactions;
use crownfall::{
    bomber::{self, Arena, End, Tile},
    chess::{Chess, Move, Outcome},
};
use raylib::prelude::*;
fn coordinates(rl: &RaylibHandle) -> (f32, Vector2, Pointer) {
    let s = (rl.get_screen_width() as f32 / WIDTH).min(rl.get_screen_height() as f32 / HEIGHT);
    let o = Vector2::new(
        (rl.get_screen_width() as f32 - WIDTH * s) * 0.5,
        (rl.get_screen_height() as f32 - HEIGHT * s) * 0.5,
    );
    let m = rl.get_mouse_position();
    (
        s,
        o,
        Pointer {
            point: Vector2::new((m.x - o.x) / s, (m.y - o.y) / s),
            clicked: rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT),
        },
    )
}
fn header(d: &mut impl RaylibDraw, f: &Fonts, title: &str, subtitle: &str) {
    d.clear_background(Color::new(228, 244, 253, 255));
    text(d, f, "ORBIT CLUB", 32., 25., 16., BLUE, true);
    text(d, f, title, 32., 52., 32., DARK, true);
    text(d, f, subtitle, 35., 92., 15., GREY, false);
}
fn orb(d: &mut impl RaylibDraw, x: f32, y: f32, r: f32, color: Color) {
    d.draw_circle(
        (x + 2.) as i32,
        (y + 4.) as i32,
        r,
        Color::new(60, 100, 130, 35),
    );
    d.draw_circle(x as i32, y as i32, r, color);
    d.draw_circle(
        (x - r * 0.25) as i32,
        (y - r * 0.3) as i32,
        r * 0.3,
        Color::new(255, 255, 255, 100),
    );
}
/// Space chess miniatures: projected, lit 3D meshes with depth-sorted triangles.
fn piece(d: &mut impl RaylibDraw, x: f32, y: f32, p: i8, size: f32) {
    use crownfall::{
        geometry::{box_mesh, cone, ring, sphere},
        math::V3,
    };
    let kind = p.abs();
    let mut parts = vec![(cone(0.65, 0.18, 24, 0.65), V3::new(0., -0.75, 0.))];
    parts.push((cone(0.3, 0.7, 20, 0.2), V3::new(0., -0.55, 0.)));
    match kind {
        1 => parts.push((sphere(20, 12, None), V3::new(0., 0.2, 0.))),
        2 => {
            parts.push((cone(0.5, 1.1, 20, 0.), V3::new(0., -0.1, 0.)));
            parts.push((box_mesh(0.9, 0.12, 0.4), V3::new(0., 0., 0.)));
        }
        3 => {
            parts.push((sphere(20, 12, None), V3::new(0., 0.2, 0.)));
            parts.push((ring(0.55, 0.8, 32), V3::new(0., 0.25, 0.)));
        }
        4 => {
            parts.push((box_mesh(0.8, 0.65, 0.6), V3::new(0., 0.15, 0.)));
            parts.push((box_mesh(1.4, 0.08, 0.7), V3::new(0., 0.15, 0.)));
        }
        5 => {
            parts.push((sphere(20, 12, None), V3::new(0., 0.2, 0.)));
            for i in 0..5 {
                let a = i as f32 * 1.256;
                parts.push((
                    cone(0.13, 0.5, 12, 0.),
                    V3::new(a.cos() * 0.4, 0.55, a.sin() * 0.4),
                ));
            }
        }
        _ => {
            parts.push((cone(0.45, 0.8, 24, 0.2), V3::new(0., 0., 0.)));
            parts.push((box_mesh(0.7, 0.12, 0.12), V3::new(0., 0.95, 0.)));
            parts.push((box_mesh(0.12, 0.5, 0.12), V3::new(0., 0.95, 0.)));
        }
    }
    let mut tris = vec![];
    for (mesh, offset) in parts {
        for face in mesh.indices.chunks_exact(3) {
            let points = face
                .iter()
                .map(|&i| {
                    let mut v = mesh.vertices[i].p;
                    if kind == 1 || kind == 3 || kind == 5 {
                        v = v * 0.4;
                    }
                    v + offset
                })
                .collect::<Vec<_>>();
            let normal = (points[1] - points[0])
                .cross(points[2] - points[0])
                .normalized();
            let light = (0.55 + normal.dot(V3::new(-0.4, 0.8, 0.5)) * 0.4).clamp(0.2, 1.);
            let color = if p > 0 {
                [155., 225., 255.]
            } else {
                [178., 123., 240.]
            };
            let projected = points
                .iter()
                .map(|v| {
                    Vector2::new(
                        x + size * 0.5 + (v.x + v.z * 0.35) * size * 0.37,
                        y + size * 0.57 + (-v.y + v.z * 0.3) * size * 0.37,
                    )
                })
                .collect::<Vec<_>>();
            tris.push((
                points.iter().map(|v| v.z + v.y * 0.3).sum::<f32>(),
                projected,
                Color::new(
                    (color[0] * light) as u8,
                    (color[1] * light) as u8,
                    (color[2] * light) as u8,
                    255,
                ),
            ));
        }
    }
    tris.sort_by(|a, b| a.0.total_cmp(&b.0));
    d.draw_ellipse(
        (x + size * 0.5) as i32,
        (y + size * 0.85) as i32,
        size * 0.3,
        size * 0.07,
        Color::new(10, 15, 40, 70),
    );
    for (_, v, c) in tris {
        d.draw_triangle(v[0], v[1], v[2], c);
        d.draw_triangle(v[2], v[1], v[0], c);
    }
}
pub fn draw_chess(
    d: &mut impl RaylibDraw,
    f: &Fonts,
    p: &Pointer,
    g: &Chess,
    selected: Option<usize>,
    promotion: &[Move],
    notice: &str,
    help: bool,
    cast: &Cast,
    host: Character,
    moods: [crownfall::reactions::Mood; 2],
    time: f32,
) -> Option<usize> {
    header(
        d,
        f,
        "PLANET CHESS",
        "White: YOU  /  Blue: planet host CPU     |     H: return to your ship",
    );
    d.clear_background(Color::new(12, 19, 47, 255));
    for i in 0..160 {
        d.draw_circle(
            (i * 137) % 1280,
            (i * 73) % 850,
            if i % 7 == 0 { 2. } else { 1. },
            Color::new(180, 205, 245, 200),
        );
    }
    text(d, f, "SPACE CHESS", 32., 45., 30., Color::SKYBLUE, true);
    cast.duel(d, f, host, moods, time);
    panel(d, rect(27., 123., 724., 686.));
    panel(d, rect(778., 123., 474., 686.));
    let (bx, by, step) = (70., 145., 76.);
    let legal = g.legal();
    let mut clicked = None;
    for s in 0..64 {
        let (x, y) = (bx + (s % 8) as f32 * step, by + (s / 8) as f32 * step);
        let r = rect(x, y, step, step);
        let mut color = if (s % 8 + s / 8) % 2 == 0 {
            Color::new(227, 243, 249, 255)
        } else {
            Color::new(134, 192, 217, 255)
        };
        if g.last.is_some_and(|m| m.from == s || m.to == s) {
            color = Color::new(177, 216, 170, 255);
        }
        if selected == Some(s) {
            color = Color::new(255, 215, 124, 255);
        }
        d.draw_rectangle_rec(r, color);
        if legal.iter().any(|m| Some(m.from) == selected && m.to == s) {
            d.draw_circle(
                (x + step / 2.) as i32,
                (y + step / 2.) as i32,
                13.,
                Color::new(53, 165, 150, 160),
            );
        }
        if g.board[s] != 0 {
            piece(d, x, y, g.board[s], step);
        }
        if p.clicked && r.check_collision_point_rec(p.point) {
            clicked = Some(s);
        }
    }
    for a in 0..8 {
        text(
            d,
            f,
            &((b'a' + a as u8) as char).to_string(),
            bx + a as f32 * step + 32.,
            by + step * 8. + 12.,
            16.,
            GREY,
            true,
        );
        text(
            d,
            f,
            &(8 - a).to_string(),
            bx - 25.,
            by + a as f32 * step + 28.,
            16.,
            GREY,
            true,
        );
    }
    let status = match g.outcome {
        Some(Outcome::Won(1)) => "CHECKMATE! YOU WIN",
        Some(Outcome::Won(_)) => "CPU WINS",
        Some(Outcome::Draw(_)) => "DRAW",
        None if g.turn == 1 => "YOUR MOVE",
        _ => "CPU IS THINKING...",
    };
    text(d, f, status, 804., 150., 23., BLUE, true);
    if g.in_check(g.turn) && g.outcome.is_none() {
        text(
            d,
            f,
            "Check! Protect your king.",
            804.,
            190.,
            18.,
            CORAL,
            true,
        );
    }
    wrap(d, f, notice, 804., 231., 410., 18., DARK);
    if !promotion.is_empty() {
        text(d, f, "CHOOSE YOUR PROMOTION", 804., 305., 17., BLUE, true);
        for (i, label) in ["Queen", "Rook", "Bishop", "Knight"].iter().enumerate() {
            if button(
                d,
                f,
                p,
                rect(
                    804. + (i % 2) as f32 * 200.,
                    343. + (i / 2) as f32 * 55.,
                    187.,
                    43.,
                ),
                label,
                false,
                true,
            ) {
                return Some(64 + i);
            }
        }
    } else {
        wrap(
            d,
            f,
            if help {
                "Click your white piece, then a green legal destination. Castling, en passant, check and all four promotions are supported. Checkmate wins; stalemate, repetition, the 50-move rule and dead material draw. For practice, Z undoes your turn and the CPU's reply."
            } else {
                "A quiet strategy duel among the ruins. Capture pieces, protect your king, and find checkmate. Your planet host evaluates legal moves and replies after your turn."
            },
            804.,
            320.,
            413.,
            18.,
            DARK,
        );
    }
    if !help && promotion.is_empty() {
        cast.body(d, host, moods[1], rect(974., 428., 156., 156.), time);
    }
    text(d, f, "Click pieces to move", 804., 588., 17., GREY, false);
    text(
        d,
        f,
        "Z  Undo round   /   F1  Rules",
        804.,
        620.,
        16.,
        GREY,
        false,
    );
    if button(
        d,
        f,
        p,
        rect(804., 674., 195., 46.),
        "New match / R",
        false,
        true,
    ) {
        return Some(100);
    }
    if button(
        d,
        f,
        p,
        rect(1015., 674., 207., 46.),
        "< Return to hub",
        true,
        true,
    ) {
        return Some(101);
    }
    if button(
        d,
        f,
        p,
        rect(804., 735., 418., 43.),
        if help { "Close rules" } else { "How to play" },
        false,
        true,
    ) {
        return Some(102);
    }
    clicked
}
pub fn chess(
    rl: &mut RaylibHandle,
    t: &RaylibThread,
    cast: &Cast,
    host: Character,
) -> Result<Option<i8>, Box<dyn std::error::Error>> {
    rl.set_window_title(t, &format!("Orbit Club / Chess vs {} CPU", host.name()));
    let f = Fonts::load(rl, t);
    let mut g = Chess::new();
    let mut moods = reactions::chess(&g);
    let mut seen_ply = u32::MAX;
    let mut age = 10.;
    let mut time = 0.;
    let mut selected = None;
    let mut promotion = vec![];
    let mut history = vec![];
    let mut clock = 0.;
    let mut notice = String::from("You are White. Click a piece to see its legal moves.");
    let mut help = false;
    let mut result = None;
    while !rl.window_should_close() {
        let dt = rl.get_frame_time().clamp(0., 0.1);
        time += dt;
        age += dt;
        let (scale, offset, p) = coordinates(rl);
        if rl.is_key_pressed(KeyboardKey::KEY_H) {
            return Ok(result);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F1) {
            help = !help;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_Z)
            && let Some(old) = history.pop()
        {
            g = old;
            selected = None;
            promotion.clear();
            clock = 0.;
            notice = "Round undone.".into();
        }
        let restart = rl.is_key_pressed(KeyboardKey::KEY_R);
        if g.turn == -1 && g.outcome.is_none() && !help {
            clock += dt;
            if clock > 0.55 {
                if let Some(m) = g.cpu() {
                    notice = format!(
                        "{}: {} to {}. Your move!",
                        host.name(),
                        crownfall::chess::name(m.from),
                        crownfall::chess::name(m.to)
                    );
                    g.play(m);
                }
                clock = 0.;
            }
        }
        if let Some(out) = g.outcome {
            if result != Some(1) {
                result = Some(match out {
                    Outcome::Won(s) => s,
                    Outcome::Draw(_) => 0,
                });
            }
            notice = match out {
                Outcome::Won(1) => "You found checkmate! Return to your ship or play again.".into(),
                Outcome::Won(_) => format!("{} found checkmate. Try a new opening!", host.name()),
                Outcome::Draw(s) => format!("Draw: {s}."),
            };
        }
        if seen_ply != g.ply {
            moods = reactions::chess(&g);
            seen_ply = g.ply;
            age = 0.;
        }
        let shown = reactions::pulse(moods, age, if g.turn == 1 { 1 } else { 0 });
        let action = {
            let mut d = rl.begin_drawing(t);
            d.clear_background(EDGE);
            let mut d = d.begin_mode2D(Camera2D {
                offset,
                target: Vector2::zero(),
                rotation: 0.,
                zoom: scale,
            });
            draw_chess(
                &mut d, &f, &p, &g, selected, &promotion, &notice, help, cast, host, shown, time,
            )
        };
        if restart || action == Some(100) {
            g = Chess::new();
            history.clear();
            promotion.clear();
            selected = None;
            notice = "New match. White moves first.".into();
            clock = 0.;
        } else if action == Some(101) {
            return Ok(result);
        } else if action == Some(102) {
            help = !help;
        } else if !help
            && g.turn == 1
            && g.outcome.is_none()
            && let Some(s) = action
        {
            if !promotion.is_empty() {
                if (64..68).contains(&s) {
                    history.push(g.clone());
                    g.play(promotion[s - 64]);
                    promotion.clear();
                    selected = None;
                    clock = 0.;
                }
            } else if s < 64 {
                let choices: Vec<_> = g
                    .legal()
                    .into_iter()
                    .filter(|m| Some(m.from) == selected && m.to == s)
                    .collect();
                if choices.len() > 1 {
                    promotion = choices;
                } else if let Some(&m) = choices.first() {
                    history.push(g.clone());
                    g.play(m);
                    selected = None;
                    clock = 0.;
                    notice = format!("{} is planning a reply...", host.name());
                } else {
                    selected = if g.board[s] > 0 { Some(s) } else { None };
                }
            }
        }
    }
    Ok(result)
}
pub fn draw_bomber(
    d: &mut impl RaylibDraw,
    f: &Fonts,
    p: &Pointer,
    a: &Arena,
    paused: bool,
    cast: &Cast,
    time: f32,
) -> Option<u8> {
    header(
        d,
        f,
        "CINDER / BLAST ORBIT",
        "Clear crates, dodge the blast, outplay EMBER CPU.  |  H: return to your ship",
    );
    cast.duel(d, f, Character::Ember, reactions::bomber(a), time);
    panel(d, rect(25., 123., 790., 690.));
    panel(d, rect(837., 123., 415., 690.));
    let (bx, by, s) = (58., 165., 55.);
    for c in 0..bomber::N {
        let (x, y) = (
            bx + (c % bomber::W) as f32 * s,
            by + (c / bomber::W) as f32 * s,
        );
        d.draw_rectangle_rec(
            rect(x, y, s - 1., s - 1.),
            if (c % bomber::W + c / bomber::W).is_multiple_of(2) {
                Color::new(232, 243, 247, 255)
            } else {
                Color::new(218, 235, 242, 255)
            },
        );
        match a.tiles[c] {
            Tile::Stone => {
                d.draw_rectangle_rounded(
                    rect(x + 2., y + 2., s - 4., s - 4.),
                    0.2,
                    8,
                    Color::new(101, 139, 162, 255),
                );
                d.draw_rectangle_rounded(
                    rect(x + 5., y + 4., s - 10., s - 15.),
                    0.15,
                    8,
                    Color::new(144, 178, 197, 255),
                );
            }
            Tile::Crate => {
                d.draw_rectangle_rounded(
                    rect(x + 3., y + 3., s - 6., s - 6.),
                    0.2,
                    8,
                    Color::new(218, 145, 100, 255),
                );
                d.draw_rectangle_lines_ex(
                    rect(x + 9., y + 9., s - 18., s - 18.),
                    3.,
                    Color::new(255, 204, 145, 255),
                );
                d.draw_line_ex(
                    Vector2::new(x + 9., y + 9.),
                    Vector2::new(x + s - 9., y + s - 9.),
                    3.,
                    Color::new(249, 194, 133, 255),
                );
            }
            _ => {}
        }
        if a.pickups[c] {
            orb(
                d,
                x + s / 2.,
                y + s / 2.,
                13.,
                Color::new(117, 200, 136, 255),
            );
            text(d, f, "+", x + 20., y + 15., 24., WHITE, true);
        }
    }
    for flame in &a.flames {
        let x = bx + (flame.cell % bomber::W) as f32 * s;
        let y = by + (flame.cell / bomber::W) as f32 * s;
        d.draw_rectangle_rounded(
            rect(x + 1., y + 1., s - 2., s - 2.),
            0.5,
            8,
            Color::new(255, 166, 74, 230),
        );
        orb(
            d,
            x + s / 2.,
            y + s / 2.,
            16.,
            Color::new(255, 233, 130, 255),
        );
    }
    for b in &a.bombs {
        let x = bx + (b.cell % bomber::W) as f32 * s + s / 2.;
        let y = by + (b.cell / bomber::W) as f32 * s + s / 2.;
        orb(
            d,
            x,
            y,
            17. + (b.fuse * 12.).sin() * 1.5,
            Color::new(35, 40, 58, 255),
        );
        d.draw_line_ex(
            Vector2::new(x + 4., y - 13.),
            Vector2::new(x + 9., y - 23.),
            3.,
            DARK,
        );
        d.draw_circle((x + 9.) as i32, (y - 23.) as i32, 4., Color::GOLD);
    }
    for who in 0..2 {
        if !a.alive[who] {
            continue;
        }
        let c = a.pawns[who];
        let x = bx + (c % bomber::W) as f32 * s;
        let y = by + (c / bomber::W) as f32 * s;
        let color = if who == 0 { BLUE } else { CORAL };
        orb(d, x + s / 2., y + s / 2., s * 0.32, color);
        d.draw_rectangle_rounded(
            rect(x + s * 0.22, y + s * 0.27, s * 0.56, s * 0.22),
            0.4,
            8,
            Color::new(24, 37, 65, 255),
        );
        for eye in [0.4, 0.6] {
            d.draw_circle((x + s * eye) as i32, (y + s * 0.38) as i32, 2., WHITE);
        }
    }
    let status = match a.outcome {
        Some(End::Win(0)) => "YOU WIN!",
        Some(End::Win(_)) => "EMBER WINS",
        Some(End::Draw) => "DRAW",
        None if paused => "PAUSED",
        _ => "YOU vs EMBER CPU",
    };
    text(d, f, status, 863., 151., 24., BLUE, true);
    text(
        d,
        f,
        &format!(
            "TIME   {}:{:02}",
            a.remaining.ceil() as i32 / 60,
            a.remaining.ceil() as i32 % 60
        ),
        863.,
        201.,
        26.,
        DARK,
        true,
    );
    wrap(
        d,
        f,
        "WASD / arrows to move.\nSpace places a bomb.\nP pauses the match.",
        863.,
        259.,
        350.,
        20.,
        DARK,
    );
    wrap(
        d,
        f,
        "Two bombs at a time. A 2.2-second fuse, then a cross-shaped blast. Stone stops it; crates absorb one hit. Bombs trigger each other!",
        863.,
        361.,
        350.,
        17.,
        GREY,
    );
    wrap(
        d,
        f,
        "Green orbs extend your blast range. Both players use the same bombs and movement speed. Survive while Ember is caught in a blast to win. A simultaneous knockout or the 2-minute limit is a draw.",
        863.,
        473.,
        350.,
        17.,
        GREY,
    );
    if button(
        d,
        f,
        p,
        rect(863., 653., 361., 43.),
        if paused { "Resume / P" } else { "Pause / P" },
        false,
        a.outcome.is_none(),
    ) {
        return Some(1);
    }
    if button(
        d,
        f,
        p,
        rect(863., 710., 171., 43.),
        "Rematch / R",
        false,
        true,
    ) {
        return Some(2);
    }
    if button(
        d,
        f,
        p,
        rect(1047., 710., 177., 43.),
        "< Hub / H",
        true,
        true,
    ) {
        return Some(3);
    }
    text(
        d,
        f,
        &format!("YOUR RANGE: {}    CRATES: {}", a.range[0], a.destroyed),
        863.,
        774.,
        14.,
        GREY,
        true,
    );
    None
}
pub fn bomber(
    rl: &mut RaylibHandle,
    t: &RaylibThread,
    cast: &Cast,
) -> Result<Option<i8>, Box<dyn std::error::Error>> {
    rl.set_window_title(t, "Orbit Club / Blast Orbit vs Ember");
    let f = Fonts::load(rl, t);
    let mut a = Arena::new();
    let mut time = 0.;
    let mut paused = false;
    let mut clock = 0.;
    let mut result = None;
    while !rl.window_should_close() {
        if rl.is_key_pressed(KeyboardKey::KEY_H) {
            return Ok(result);
        }
        let dt = rl.get_frame_time().clamp(0., 0.1);
        time += dt;
        let (scale, offset, p) = coordinates(rl);
        if rl.is_key_pressed(KeyboardKey::KEY_P) || rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            paused = !paused;
        }
        let restart = rl.is_key_pressed(KeyboardKey::KEY_R);
        if !paused && a.outcome.is_none() {
            clock += dt;
            if clock >= 0.17 {
                let c = a.pawns[0];
                let next =
                    if rl.is_key_down(KeyboardKey::KEY_W) || rl.is_key_down(KeyboardKey::KEY_UP) {
                        c.checked_sub(bomber::W)
                    } else if rl.is_key_down(KeyboardKey::KEY_S)
                        || rl.is_key_down(KeyboardKey::KEY_DOWN)
                    {
                        c.checked_add(bomber::W).filter(|&n| n < bomber::N)
                    } else if rl.is_key_down(KeyboardKey::KEY_A)
                        || rl.is_key_down(KeyboardKey::KEY_LEFT)
                    {
                        c.checked_sub(1)
                    } else if rl.is_key_down(KeyboardKey::KEY_D)
                        || rl.is_key_down(KeyboardKey::KEY_RIGHT)
                    {
                        c.checked_add(1).filter(|&n| n < bomber::N)
                    } else {
                        None
                    };
                if let Some(n) = next {
                    a.walk(0, n);
                    clock = 0.;
                }
            }
            if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                a.plant(0);
            }
            a.update(dt);
        }
        if let Some(out) = a.outcome
            && result != Some(1)
        {
            result = Some(match out {
                End::Win(0) => 1,
                End::Win(_) => -1,
                End::Draw => 0,
            });
        }
        let action = {
            let mut d = rl.begin_drawing(t);
            d.clear_background(EDGE);
            let mut d = d.begin_mode2D(Camera2D {
                offset,
                target: Vector2::zero(),
                rotation: 0.,
                zoom: scale,
            });
            draw_bomber(&mut d, &f, &p, &a, paused, cast, time)
        };
        match action {
            Some(1) => paused = !paused,
            Some(3) => return Ok(result),
            _ => {}
        }
        if restart || action == Some(2) {
            a = Arena::new();
            paused = false;
            clock = 0.;
        }
    }
    Ok(result)
}
