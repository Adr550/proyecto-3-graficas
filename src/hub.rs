//! Arrival-gated planet arcade, character hosts, and assisted third-person navigation.
#![allow(clippy::too_many_arguments)]
use crate::{
    arcade,
    characters::{Cast, Character},
    connect_screen, quoridor, studio,
    view::{self, *},
};
use crownfall::{
    flight::{App, Input, Mode},
    math::V3,
    reactions::Mood,
    renderer::Renderer,
    world::DEFINITIONS,
};
use raylib::prelude::*;
use std::{fs, path::PathBuf};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Activity {
    Quoridor,
    Chess,
    Blast,
    Connect4,
    Beat,
}
impl Activity {
    fn label(self) -> &'static str {
        match self {
            Self::Quoridor => "QUORIDOR",
            Self::Chess => "CHESS",
            Self::Blast => "BLAST ORBIT",
            Self::Connect4 => "CONNECT FOUR",
            Self::Beat => "ORBIT BEAT",
        }
    }
    fn index(self) -> usize {
        match self {
            Self::Quoridor => 0,
            Self::Chess => 1,
            Self::Blast => 2,
            Self::Connect4 => 3,
            Self::Beat => 4,
        }
    }
}
pub const ACTIVITIES: [Activity; 5] = [
    Activity::Blast,
    Activity::Beat,
    Activity::Quoridor,
    Activity::Chess,
    Activity::Connect4,
];
#[derive(Default)]
struct Progress {
    visited: [bool; 5],
    medals: [bool; 5],
}
impl Progress {
    fn path() -> PathBuf {
        let root = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        if cfg!(target_os = "macos") {
            root.join("Library/Application Support/Orbit Club/progress.txt")
        } else {
            root.join(".local/share/orbit-club/progress.txt")
        }
    }
    fn parse(s: &str) -> Self {
        let b = s.trim().as_bytes();
        Self {
            visited: std::array::from_fn(|i| b.get(i) == Some(&b'1')),
            medals: std::array::from_fn(|i| b.get(5 + i) == Some(&b'1')),
        }
    }
    fn load() -> Self {
        Self::parse(&fs::read_to_string(Self::path()).unwrap_or_default())
    }
    fn save(&self) {
        let path = Self::path();
        if let Some(p) = path.parent() {
            let _ = fs::create_dir_all(p);
        }
        let s: String = self
            .visited
            .iter()
            .chain(self.medals.iter())
            .map(|&b| if b { '1' } else { '0' })
            .collect();
        let _ = fs::write(path, s);
    }
}
fn ready(app: &App) -> Option<(usize, Activity)> {
    app.docked().map(|i| (i, ACTIVITIES[i]))
}
fn make_app() -> App {
    let mut app = App::new();
    app.mode = Mode::Flight;
    app.selected = Some(4);
    let p = app.world.planets[4].pos;
    app.ship.position = p + V3::new(0., 45., 170.);
    app.ship.aim(p - app.ship.position);
    app.camera = app.desired_camera();
    app
}
fn scene(app: &App, r: &mut Renderer) {
    r.begin(app.camera, app.world.elapsed as f32);
    app.world
        .draw(r, true, app.selected, Some(app.ship.model()));
    r.finish();
}
fn radar(d: &mut impl RaylibDraw, p: &Pointer, app: &App) -> Option<u8> {
    let c = Vector2::new(835., 691.);
    let scale = 0.19;
    d.draw_circle(c.x as i32, c.y as i32, 76., Color::new(255, 255, 255, 235));
    d.draw_circle_lines(c.x as i32, c.y as i32, 76., EDGE);
    d.draw_circle(c.x as i32, c.y as i32, 6., Color::new(252, 196, 86, 255));
    for (i, planet) in app.world.planets.iter().enumerate() {
        let q = Vector2::new(c.x + planet.pos.x * scale, c.y + planet.pos.z * scale);
        let color = Character::planet(i).color();
        d.draw_circle(q.x as i32, q.y as i32, 6., color);
        if app.selected == Some(i) {
            d.draw_circle_lines(q.x as i32, q.y as i32, 10., BLUE);
        }
        if p.clicked && q.distance(p.point) < 12. {
            return Some(i as u8);
        }
    }
    let ship = Vector2::new(
        c.x + app.ship.position.x.clamp(-350., 350.) * scale,
        c.y + app.ship.position.z.clamp(-350., 350.) * scale,
    );
    let forward = app.ship.forward();
    d.draw_circle(ship.x as i32, ship.y as i32, 4., BLUE);
    d.draw_line_ex(
        ship,
        Vector2::new(ship.x + forward.x * 13., ship.y + forward.z * 13.),
        3.,
        BLUE,
    );
    None
}
fn draw(
    d: &mut impl RaylibDraw,
    f: &Fonts,
    p: &Pointer,
    app: &App,
    r: &Renderer,
    tex: &Texture2D,
    progress: &Progress,
    help: bool,
    cast: &Cast,
) -> Option<u8> {
    let inert = Pointer {
        point: p.point,
        clicked: p.clicked && !help,
    };
    let pointer = &inert;
    let time = app.world.elapsed as f32;
    d.clear_background(Color::new(228, 244, 253, 255));
    cast.portrait(
        d,
        Character::Fer,
        if app.travelling() {
            Mood::Thinking
        } else {
            Mood::Idle
        },
        Vector2::new(51., 54.),
        34.,
        time,
    );
    text(d, f, "ORBIT CLUB", 99., 25., 33., BLUE, true);
    text(
        d,
        f,
        "Fer's ship / five worlds, five CPU challenges",
        100.,
        69.,
        15.,
        GREY,
        false,
    );
    panel(d, rect(590., 23., 302., 67.));
    text(
        d,
        f,
        &format!(
            "{} / 5 WORLDS VISITED",
            progress.visited.iter().filter(|&&v| v).count()
        ),
        608.,
        36.,
        16.,
        DARK,
        true,
    );
    text(
        d,
        f,
        &format!(
            "{} / 5 VICTORY STARS",
            progress.medals.iter().filter(|&&v| v).count()
        ),
        608.,
        61.,
        14.,
        BLUE,
        true,
    );
    if button(
        d,
        f,
        pointer,
        rect(916., 30., 155., 45.),
        if app.mode == Mode::Overview {
            "Take flight / Tab"
        } else {
            "System map / Tab"
        },
        false,
        true,
    ) {
        return Some(10);
    }
    if button(
        d,
        f,
        pointer,
        rect(1086., 30., 165., 45.),
        "Flight guide / F1",
        false,
        true,
    ) {
        return Some(11);
    }
    let status = if app.travelling() {
        "AUTOPILOT / A-D or the arrows take over manual steering"
    } else if ready(app).is_some() {
        "DOCKED! Enter launches this world's game"
    } else if app.mode == Mode::Flight {
        "A-D TURN / W-S THRUST / arrows aim / R faces your destination"
    } else {
        "MAP / click a world to mark it / Tab returns to flight / G warps"
    };
    text(d, f, status, 34., 126., 17., DARK, true);
    d.draw_rectangle_rounded(rect(20., 160., 918., 633.), 0.04, 10, WHITE);
    d.draw_texture_pro(
        tex,
        rect(0., 0., r.width as f32, r.height as f32),
        view::SCENE,
        Vector2::zero(),
        0.,
        WHITE,
    );
    if app.mode == Mode::Flight {
        let center = Vector2::new(
            view::SCENE.x + view::SCENE.width * 0.5,
            view::SCENE.y + view::SCENE.height * 0.5,
        );
        d.draw_circle_lines(
            center.x as i32,
            center.y as i32,
            7.,
            Color::new(180, 230, 255, 160),
        );
        text(
            d,
            f,
            &format!(
                "SPEED {:03.0}  /  RIGHT-DRAG STEER  /  G WARP",
                app.ship.velocity.length()
            ),
            view::SCENE.x + 20.,
            view::SCENE.y + 18.,
            14.,
            Color::SKYBLUE,
            true,
        );
    }
    let warp = app.warp_strength();
    if warp > 0.001 {
        let area = view::SCENE;
        let center = Vector2::new(area.x + area.width * 0.5, area.y + area.height * 0.5);
        d.draw_rectangle_rec(area, Color::new(40, 25, 110, (warp * 45.) as u8));
        for i in 0..110 {
            let angle = i as f32 * 2.399963;
            let dir = Vector2::new(angle.cos(), angle.sin());
            let edge = (area.width * 0.49 / dir.x.abs().max(0.001))
                .min(area.height * 0.49 / dir.y.abs().max(0.001));
            let phase = (time * 1.7 + i as f32 * 0.618034).fract();
            let distance = 0.08 + phase * 0.9;
            let a = center + dir * (edge * distance);
            let b = center + dir * (edge * (distance + 0.03 + warp * 0.18).min(0.99));
            d.draw_line_ex(
                a,
                b,
                1. + warp * 2.,
                Color::new(
                    if i % 3 == 0 { 210 } else { 100 },
                    210,
                    255,
                    (warp * (80. + phase * 160.)) as u8,
                ),
            );
        }
        text(
            d,
            f,
            "WARP DRIVE",
            area.x + area.width * 0.5 - 62.,
            area.y + 30.,
            20.,
            Color::SKYBLUE,
            true,
        );
    }
    let over_controls = rect(34., 631., 361., 146.).check_collision_point_rec(pointer.point)
        || rect(430., 690., 260., 85.).check_collision_point_rec(pointer.point)
        || (app.mode == Mode::Flight && Vector2::new(835., 691.).distance(pointer.point) < 78.);
    let map_click = pointer.clicked && !over_controls;
    for (i, planet) in app.world.planets.iter().enumerate() {
        let q = r.project(planet.pos + V3::Y * (planet.def.radius + 9.));
        if q.w > 0.
            && q.x > 20.
            && q.x < r.width as f32 - 126.
            && q.y > 20.
            && q.y < r.height as f32 - 50.
        {
            let x = view::SCENE.x + q.x;
            let y = view::SCENE.y + q.y;
            let label = rect(x - 8., y - 3., 134., 43.);
            d.draw_rectangle_rounded(label, 0.3, 8, Color::new(255, 255, 255, 230));
            text(d, f, planet.def.name, x, y, 15., DARK, true);
            text(d, f, ACTIVITIES[i].label(), x, y + 21., 10., BLUE, true);
            if map_click && label.check_collision_point_rec(pointer.point) {
                return Some(i as u8);
            }
        }
    }
    // The rendered planet itself can be selected, not only its label.
    if app.mode == Mode::Overview
        && map_click
        && view::SCENE.check_collision_point_rec(pointer.point)
    {
        let x = (pointer.point.x - view::SCENE.x) as usize;
        let y = (pointer.point.y - view::SCENE.y) as usize;
        if x < r.width && y < r.height {
            let id = r.ids[y * r.width + x];
            if (2..=6).contains(&id) {
                return Some((id - 2) as u8);
            }
        }
    }
    text(d, f, "MEET THE PLANET HOSTS", 960., 127., 16., BLUE, true);
    for (i, activity) in ACTIVITIES.iter().enumerate() {
        let y = 166. + i as f32 * 91.;
        let card = rect(958., y, 293., 80.);
        panel(d, card);
        let selected = app.selected == Some(i);
        if selected {
            d.draw_rectangle_rounded_lines_ex(card, 0.12, 10, 2., BLUE);
        }
        let c = Character::planet(i);
        cast.portrait(
            d,
            c,
            if selected && app.travelling() {
                Mood::Thinking
            } else {
                Mood::Idle
            },
            Vector2::new(990., y + 37.),
            25.,
            time,
        );
        text(
            d,
            f,
            &format!("{} {}", i + 1, DEFINITIONS[i].name),
            1024.,
            y + 8.,
            18.,
            DARK,
            true,
        );
        text(d, f, c.name(), 1024., y + 32., 12., GREY, false);
        text(d, f, activity.label(), 1024., y + 56., 11., BLUE, true);
        if progress.visited[i] {
            d.draw_circle(1231, y as i32 + 17, 4., Color::new(92, 183, 136, 255));
        }
        if button(
            d,
            f,
            pointer,
            rect(1147., y + 38., 91., 31.),
            "Warp / G",
            selected,
            true,
        ) {
            return Some(20 + i as u8);
        }
        if pointer.clicked && card.check_collision_point_rec(pointer.point) {
            return Some(i as u8);
        }
    }
    let selected = app.selected.unwrap_or_else(|| app.nearest());
    let host = Character::planet(selected);
    panel(d, rect(34., 631., 361., 146.));
    cast.body(
        d,
        host,
        if ready(app).is_some() {
            Mood::Happy
        } else {
            Mood::Idle
        },
        rect(39., 632., 138., 138.),
        time,
    );
    text(d, f, host.name(), 183., 651., 20., host.color(), true);
    wrap(
        d,
        f,
        if ready(app).is_some() {
            "Welcome, Fer! Ready for a friendly challenge?"
        } else {
            "Come visit my world. I'll be your CPU rival!"
        },
        183.,
        683.,
        190.,
        17.,
        DARK,
    );
    if app.mode == Mode::Flight
        && let Some(a) = radar(d, pointer, app)
    {
        return Some(a);
    }
    panel(d, rect(958., 635., 293., 158.));
    if let Some((i, a)) = ready(app) {
        text(
            d,
            f,
            &format!("DOCKED / {}", DEFINITIONS[i].name.to_uppercase()),
            975.,
            651.,
            16.,
            BLUE,
            true,
        );
        text(
            d,
            f,
            &format!("{} vs {}", a.label(), Character::planet(i).name()),
            975.,
            684.,
            12.,
            DARK,
            true,
        );
        if button(
            d,
            f,
            pointer,
            rect(975., 722., 258., 48.),
            "Play / Enter",
            true,
            true,
        ) {
            return Some(12);
        }
    } else {
        wrap(
            d,
            f,
            if app.travelling() {
                "Following the planet's orbit. Turning manually cancels autopilot."
            } else {
                "Explore freely. Select a world with 1-5. G warps there; Enter plays when nearby."
            },
            975.,
            652.,
            255.,
            17.,
            DARK,
        );
    }
    if button(
        d,
        f,
        pointer,
        rect(430., 690., 260., 36.),
        "Character Gallery / F2",
        false,
        true,
    ) {
        return Some(15);
    }
    if button(
        d,
        f,
        pointer,
        rect(430., 738., 260., 36.),
        "Face destination / R",
        false,
        app.selected.is_some(),
    ) {
        return Some(14);
    }
    text(
        d,
        f,
        "W-S thrust  /  A-D turn  /  arrows aim  /  Q-E height  /  Z-C strafe  /  Shift boost  /  X brake",
        31.,
        812.,
        14.,
        GREY,
        false,
    );
    if help {
        d.draw_rectangle_rec(rect(0., 0., WIDTH, HEIGHT), Color::new(36, 72, 104, 100));
        panel(d, rect(230., 145., 820., 570.));
        text(d, f, "FLY, MEET, PLAY", 265., 178., 30., BLUE, true);
        wrap(
            d,
            f,
            "Explore freely from the start. Click a world or press 1-5 to mark a destination. Fly there yourself, or press G for an optional warp. Enter opens its game when nearby. H returns from a game.",
            266.,
            238.,
            744.,
            20.,
            DARK,
        );
        wrap(
            d,
            f,
            "W/S flies forward/backward. A/D or Left/Right turn; Up/Down aim the nose. Q/E changes height; Z/C strafes. Hold right mouse to steer. Shift boosts; X brakes. Tab opens/closes the map without moving the ship. Wheel adjusts camera distance. O holds a planet orbit; manual input releases it.",
            266.,
            354.,
            744.,
            20.,
            DARK,
        );
        wrap(
            d,
            f,
            "Aurelia: Captain Truffle's Quoridor. Bloom: Mochi's Connect Four. Relic: Bleep the space robot hosts Orbit Beat. Prism: Lumen, the faceless star mage. Cinder: Ember's Blast Orbit. Their portraits react to your actual moves and get nervous when you threaten a win.",
            266.,
            482.,
            744.,
            20.,
            DARK,
        );
        if button(
            d,
            f,
            p,
            rect(445., 639., 390., 47.),
            "Let's explore / F1",
            true,
            true,
        ) {
            return Some(11);
        }
    }
    None
}
pub fn run(
    rl: &mut RaylibHandle,
    t: &RaylibThread,
    cast: &Cast,
) -> Result<(), Box<dyn std::error::Error>> {
    let f = Fonts::load(rl, t);
    let mut app = make_app();
    let mut progress = Progress::load();
    app.visited = progress.visited;
    let mut help = false;
    let mut r = Renderer::new(910, 625);
    let img = Image::gen_image_color(910, 625, Color::WHITE);
    let mut tex = rl.load_texture_from_image(t, &img)?;
    tex.set_texture_filter(t, TextureFilter::TEXTURE_FILTER_BILINEAR);
    let smoke = std::env::args().any(|s| s == "--smoke");
    let mut frames = 0;
    while !rl.window_should_close() {
        let dt = rl.get_frame_time().clamp(0., 0.1);
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
        let inside = view::SCENE.check_collision_point_rec(p.point);
        let piloting = inside
            && !rect(34., 631., 361., 146.).check_collision_point_rec(p.point)
            && !rect(430., 690., 260., 85.).check_collision_point_rec(p.point)
            && Vector2::new(835., 691.).distance(p.point) > 78.;
        let mut action = None;
        if rl.is_key_pressed(KeyboardKey::KEY_F1) {
            help = !help;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            help = false;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F) {
            rl.toggle_fullscreen();
        }
        if !help {
            if rl.is_key_pressed(KeyboardKey::KEY_F2) {
                action = Some(15);
            }
            if rl.is_key_pressed(KeyboardKey::KEY_TAB) {
                action = Some(10);
            }
            if rl.is_key_pressed(KeyboardKey::KEY_G) {
                action = Some(16);
            }
            if rl.is_key_pressed(KeyboardKey::KEY_R) {
                action = Some(14);
            }
            if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                action = Some(12);
            }
            if rl.is_key_pressed(KeyboardKey::KEY_O) {
                app.toggle_local();
            }
            for (i, k) in [
                KeyboardKey::KEY_ONE,
                KeyboardKey::KEY_TWO,
                KeyboardKey::KEY_THREE,
                KeyboardKey::KEY_FOUR,
                KeyboardKey::KEY_FIVE,
            ]
            .iter()
            .enumerate()
            {
                if rl.is_key_pressed(*k) {
                    action = Some(i as u8);
                }
            }
            let held = |k| if rl.is_key_down(k) { 1. } else { 0. };
            let delta = rl.get_mouse_delta();
            let right = piloting && rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT);
            let left = piloting && rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT);
            app.aspect = 910. / 625.;
            app.update(
                dt,
                Input {
                    forward: held(KeyboardKey::KEY_W) - held(KeyboardKey::KEY_S),
                    turn: held(KeyboardKey::KEY_D) + held(KeyboardKey::KEY_RIGHT)
                        - held(KeyboardKey::KEY_A)
                        - held(KeyboardKey::KEY_LEFT),
                    pitch: held(KeyboardKey::KEY_UP) - held(KeyboardKey::KEY_DOWN),
                    strafe: held(KeyboardKey::KEY_C) - held(KeyboardKey::KEY_Z),
                    lift: held(KeyboardKey::KEY_E) - held(KeyboardKey::KEY_Q),
                    boost: rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT),
                    brake: rl.is_key_down(KeyboardKey::KEY_X),
                    mouse_x: delta.x,
                    mouse_y: delta.y,
                    dragging: (right || left) && app.mode == Mode::Overview,
                    steering: right && app.mode == Mode::Flight,
                    scroll: if inside {
                        rl.get_mouse_wheel_move()
                    } else {
                        0.
                    },
                },
            );
            if let Some((i, _)) = ready(&app)
                && !progress.visited[i]
            {
                progress.visited[i] = true;
                if !smoke {
                    progress.save();
                }
            }
        }
        scene(&app, &mut r);
        tex.update_texture(&r.pixels)?;
        {
            let mut d = rl.begin_drawing(t);
            d.clear_background(EDGE);
            let mut d = d.begin_mode2D(Camera2D {
                offset,
                target: Vector2::zero(),
                rotation: 0.,
                zoom: scale,
            });
            if let Some(a) = draw(&mut d, &f, &p, &app, &r, &tex, &progress, help, cast)
                && (!help || a == 11)
            {
                action = Some(a);
            }
        }
        match action {
            Some(i @ 0..=4) if !help => app.selected = Some(i as usize),
            Some(i @ 20..=24) if !help => app.travel_to((i - 20) as usize),
            Some(16) if !help => {
                if let Some(i) = app.selected {
                    app.travel_to(i);
                }
            }
            Some(10) => {
                if app.mode == Mode::Overview {
                    app.start_flight(false);
                } else {
                    let selected = app.selected;
                    app.overview();
                    app.selected = selected;
                }
            }
            Some(11) => help = !help,
            Some(14) => app.face_destination(),
            Some(15) => {
                studio::run(rl, t, cast)?;
                rl.set_window_title(t, "Orbit Club / Meet your planet rivals");
            }
            Some(12) if !help => {
                if let Some((i, a)) = ready(&app) {
                    let result = match a {
                        Activity::Quoridor => {
                            quoridor::play(rl, t, cast)?.map(|w| if w { 1 } else { -1 })
                        }
                        Activity::Beat => crate::rhythm::play(rl, t, cast)?,
                        Activity::Chess => {
                            if i == 1 {
                                crate::rhythm::play(rl, t, cast)?
                            } else {
                                arcade::chess(rl, t, cast, Character::planet(i))?
                            }
                        }
                        Activity::Blast => arcade::bomber(rl, t, cast)?,
                        Activity::Connect4 => connect_screen::play(rl, t, cast)?,
                    };
                    if result == Some(1) {
                        progress.medals[a.index()] = true;
                        progress.save();
                    }
                    rl.set_window_title(t, "Orbit Club / Meet your planet rivals");
                }
            }
            _ => {}
        }
        frames += 1;
        if smoke && frames >= 120 {
            break;
        }
    }
    Ok(())
}
pub fn export(dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    use crownfall::png::write_png;
    fs::create_dir_all(dir)?;
    let mut app = make_app();
    let mut r = Renderer::new(1200, 820);
    scene(&app, &mut r);
    write_png(
        dir.join("orbit-club-system.png"),
        r.width,
        r.height,
        &r.pixels,
    )?;
    app.travel_to(4);
    for _ in 0..210 {
        app.update(1. / 60., Input::default());
    }
    scene(&app, &mut r);
    write_png(
        dir.join("orbit-club-arrival.png"),
        r.width,
        r.height,
        &r.pixels,
    )?;
    Ok(())
}
pub fn captures(
    rl: &mut RaylibHandle,
    t: &RaylibThread,
    dir: &std::path::Path,
    cast: &Cast,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::client::Client;
    use crownfall::{bomber::Arena, bot::Level, chess::Chess, scene::Scene};
    fs::create_dir_all(dir)?;
    let f = Fonts::load(rl, t);
    let p = Pointer {
        point: Vector2::new(-100., -100.),
        clicked: false,
    };
    let mut app = make_app();
    let mut r = Renderer::new(910, 625);
    let img = Image::gen_image_color(910, 625, Color::WHITE);
    let mut tex = rl.load_texture_from_image(t, &img)?;
    let progress = Progress::default();
    for stage in 0..8 {
        if stage == 1 {
            app.travel_to(4);
            for _ in 0..210 {
                app.update(1. / 60., Input::default());
            }
        }
        scene(&app, &mut r);
        tex.update_texture(&r.pixels)?;
        let mut court = Scene::new(910, 625);
        let mut client = Client::new(Level::Club);
        if stage == 5 {
            client.game.pawns[0] = 18;
            court.render(&client.game, &client.game.moves(0), None, None);
            tex.update_texture(&court.renderer.pixels)?;
        }
        for _ in 0..3 {
            let mut d = rl.begin_drawing(t);
            match stage {
                0..=2 => {
                    draw(&mut d, &f, &p, &app, &r, &tex, &progress, stage == 2, cast);
                }
                3 => {
                    let g = Chess::new();
                    arcade::draw_chess(
                        &mut d,
                        &f,
                        &p,
                        &g,
                        Some(52),
                        &[],
                        "Click a legal destination.",
                        false,
                        cast,
                        Character::Star,
                        crownfall::reactions::chess(&g),
                        0.,
                    );
                }
                4 => {
                    arcade::draw_bomber(&mut d, &f, &p, &Arena::new(), false, cast, 0.);
                }
                5 => {
                    view::draw(&mut d, &f, &client, &court, &tex, &p, cast);
                }
                6 => connect_screen::capture(&mut d, &f, &p, cast),
                _ => {
                    d.clear_background(Color::new(232, 244, 250, 255));
                    for row in 0..6 {
                        let c = [
                            Character::Fer,
                            Character::Pig,
                            Character::Cat,
                            Character::Star,
                            Character::Ember,
                            Character::Prism,
                        ][row];
                        text(
                            &mut d,
                            &f,
                            c.name(),
                            20.,
                            15. + row as f32 * 138.,
                            16.,
                            c.color(),
                            true,
                        );
                        for col in 0..6 {
                            let mood = [
                                Mood::Idle,
                                Mood::Thinking,
                                Mood::Nervous,
                                Mood::Happy,
                                Mood::Sad,
                                Mood::Surprised,
                            ][col];
                            cast.body(
                                &mut d,
                                c,
                                mood,
                                rect(155. + col as f32 * 175., row as f32 * 138., 130., 130.),
                                0.,
                            );
                        }
                    }
                }
            }
        }
        let name = [
            "hub",
            "arrival",
            "guide",
            "chess",
            "blast",
            "quoridor",
            "connect-four",
            "cast",
        ][stage];
        rl.take_screenshot(
            t,
            dir.join(format!("orbit-club-{name}.png"))
                .to_string_lossy()
                .as_ref(),
        );
    }
    println!("Saved eight native captures to {}", dir.display());
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hub_starts_in_free_flight_with_chase_camera() {
        let app = make_app();
        assert_eq!(app.mode, Mode::Flight);
        assert!(!app.travelling());
        assert_eq!(app.ship.local, None);
        assert!((app.camera.eye - app.ship.position).length() < 50.);
        assert_eq!(ready(&app), None);
    }
    #[test]
    fn arrival_gates_game_launch() {
        let mut app = make_app();
        assert_eq!(ready(&app), None);
        app.travel_to(4);
        assert_eq!(ready(&app), None);
        for _ in 0..210 {
            app.update(1. / 60., Input::default());
        }
        assert_eq!(ready(&app), Some((4, Activity::Connect4)));
        app.overview();
        assert_eq!(ready(&app), None);
    }
    #[test]
    fn every_game_and_world_has_a_host() {
        assert_eq!(ACTIVITIES.len(), DEFINITIONS.len());
        for a in [
            Activity::Chess,
            Activity::Blast,
            Activity::Quoridor,
            Activity::Connect4,
        ] {
            assert!(ACTIVITIES.contains(&a));
        }
        assert_eq!(Character::planet(4), Character::Cat);
    }
    #[test]
    fn old_progress_preserves_stars() {
        let p = Progress::parse("10101111");
        assert_eq!(p.medals, [true, true, true, false, false]);
        assert_eq!(p.visited, [true, false, true, false, true]);
        let p = Progress::parse("1010111111");
        assert!(p.medals.iter().all(|&v| v));
    }
}
