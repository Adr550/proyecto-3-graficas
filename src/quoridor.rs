use crate::characters::Cast;
use crate::{client, view};
use client::{Client, Command, Modal, Tool};
use crownfall::{
    bot::{self, Level},
    game::*,
    png::write_png,
    scene::{Preview, Scene},
};
use raylib::prelude::*;
use std::{fs, path::PathBuf};
fn snapshots() -> PathBuf {
    let e = std::env::current_exe().unwrap_or_default();
    if e.parent().is_some_and(|p| p.ends_with("MacOS")) {
        e.ancestors().nth(4).unwrap().join("snapshots")
    } else {
        std::env::current_dir()
            .unwrap_or_default()
            .join("snapshots")
    }
}
fn simulation() {
    let (mut wins, mut plies) = ([0u32; 2], 0u32);
    for i in 0..30 {
        let mut g = Game::new();
        for p in 0..250 {
            if g.winner.is_some() {
                break;
            }
            let a = bot::choose(
                &g,
                if (i + p) % 5 == 0 {
                    Level::Casual
                } else {
                    Level::Club
                },
            );
            g.apply(a).unwrap();
        }
        assert!(g.winner.is_some(), "Match {i} stalled");
        wins[g.winner.unwrap()] += 1;
        plies += g.ply;
    }
    println!(
        "30 completed legal matches; results {wins:?}; average {:0.1} turns.",
        plies as f32 / 30.
    );
}
pub fn command_line() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.iter().any(|x| x == "--simulate") {
        simulation();
        return Ok(());
    }
    if let Some(i) = args.iter().position(|x| x == "--render") {
        let dir = PathBuf::from(args.get(i + 1).map(String::as_str).unwrap_or("renders"));
        fs::create_dir_all(&dir)?;
        let mut scene = Scene::new(1200, 820);
        let mut g = Game::new();
        scene.render(&g, &g.moves(0), None, None);
        write_png(
            dir.join("orbit-path-board.png"),
            1200,
            820,
            &scene.renderer.pixels,
        )?;
        g.apply(Action::Place {
            x: 3,
            y: 5,
            axis: Axis::Horizontal,
        })?;
        g.apply(Action::Place {
            x: 4,
            y: 2,
            axis: Axis::Vertical,
        })?;
        scene.render(
            &g,
            &g.moves(0),
            None,
            Some(Preview {
                wall: Wall {
                    x: 4,
                    y: 4,
                    axis: Axis::Horizontal,
                    owner: 0,
                },
                valid: true,
            }),
        );
        write_png(
            dir.join("orbit-path-barriers.png"),
            1200,
            820,
            &scene.renderer.pixels,
        )?;
        println!("Saved native 3D renders to {}", dir.display());
        return Ok(());
    }
    unreachable!("Use --simulate or --render")
}
pub fn play(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    cast: &Cast,
) -> Result<Option<bool>, Box<dyn std::error::Error>> {
    rl.set_window_title(
        thread,
        "Orbit Club / Quoridor vs Truffle / H: return to hub",
    );
    let fonts = view::Fonts::load(rl, thread);
    let mut client = Client::new(Level::Club);
    let mut scene = Scene::new(910, 625);
    let image = Image::gen_image_color(910, 625, Color::WHITE);
    let mut texture = rl.load_texture_from_image(thread, &image)?;
    texture.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
    let mut save = false;
    let mut result = None;
    while !rl.window_should_close() {
        if rl.is_key_pressed(KeyboardKey::KEY_H) {
            return Ok(result);
        }
        let dt = rl.get_frame_time().clamp(0., 0.12);
        let (w, h) = (rl.get_screen_width() as f32, rl.get_screen_height() as f32);
        let scale = (w / view::WIDTH).min(h / view::HEIGHT);
        let offset = Vector2::new(
            (w - view::WIDTH * scale) * 0.5,
            (h - view::HEIGHT * scale) * 0.5,
        );
        let mouse = rl.get_mouse_position();
        let point = Vector2::new((mouse.x - offset.x) / scale, (mouse.y - offset.y) / scale);
        let inside = view::SCENE.check_collision_point_rec(point);
        let local = Vector2::new(point.x - view::SCENE.x, point.y - view::SCENE.y);
        let delta = rl.get_mouse_delta();
        let clicked = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        if rl.is_key_pressed(KeyboardKey::KEY_F) {
            rl.toggle_fullscreen();
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F12) {
            save = true;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F1) || rl.is_key_pressed(KeyboardKey::KEY_SLASH) {
            client.command(Command::Rules);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            if client.modal != Modal::None {
                client.command(Command::Close);
            } else {
                client.command(Command::MoveTool);
            }
        }
        if client.modal == Modal::None {
            if rl.is_key_pressed(KeyboardKey::KEY_M) {
                client.command(Command::MoveTool);
            }
            if rl.is_key_pressed(KeyboardKey::KEY_B) {
                client.command(Command::BarrierTool);
            }
            if rl.is_key_pressed(KeyboardKey::KEY_R) || rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                client.command(Command::Rotate);
            }
            if rl.is_key_pressed(KeyboardKey::KEY_Z) {
                client.command(Command::Undo);
            }
            if rl.is_key_pressed(KeyboardKey::KEY_V) {
                scene.reset_view();
            }
            if inside {
                scene.distance =
                    (scene.distance * (-rl.get_mouse_wheel_move() * 0.10).exp()).clamp(135., 280.);
                if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT)
                    && client.tool == Tool::Move
                    || rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_MIDDLE)
                {
                    scene.eye_yaw -= delta.x * 0.005;
                    scene.eye_pitch = (scene.eye_pitch + delta.y * 0.004).clamp(0.55, 1.45);
                }
                client.hover_cell = scene.pick_cell(local.x, local.y);
                client.hover_wall = scene.pick_wall(local.x, local.y);
                if clicked && client.human_ready() {
                    match client.tool {
                        Tool::Move => {
                            if let Some(c) = client.hover_cell {
                                client.human_action(Action::Move(c));
                            }
                        }
                        Tool::Barrier => {
                            if let Some((x, y)) = client.hover_wall {
                                client.human_action(Action::Place {
                                    x,
                                    y,
                                    axis: client.axis,
                                });
                            }
                        }
                    }
                }
                if client.tool == Tool::Barrier
                    && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT)
                {
                    client.command(Command::Rotate);
                }
            } else {
                client.hover_cell = None;
                client.hover_wall = None;
            }
        }
        client.update(dt);
        if let Some(w) = client.game.winner
            && result != Some(true)
        {
            result = Some(w == 0);
        }
        scene.time += dt;
        let positions = client.ship_positions();
        for (player, &pos) in positions.iter().enumerate() {
            let delta = pos - scene.ship_positions[player];
            if delta.x.abs() + delta.z.abs() > 0.001 {
                scene.ship_yaws[player] = (-delta.x).atan2(-delta.z);
            }
            scene.ship_positions[player] = pos;
        }
        let legal = if client.hints && client.human_ready() {
            client.game.moves(0)
        } else {
            vec![]
        };
        let preview = if client.tool == Tool::Barrier && client.human_ready() {
            client.hover_wall.map(|(x, y)| Preview {
                wall: Wall {
                    x,
                    y,
                    axis: client.axis,
                    owner: 0,
                },
                valid: client
                    .game
                    .validate(Action::Place {
                        x,
                        y,
                        axis: client.axis,
                    })
                    .is_ok(),
            })
        } else {
            None
        };
        scene.render(&client.game, &legal, client.hover_cell, preview);
        texture.update_texture(&scene.renderer.pixels)?;
        let pointer = view::Pointer { point, clicked };
        let command = {
            let mut d = rl.begin_drawing(thread);
            d.clear_background(Color::new(226, 242, 252, 255));
            let camera = Camera2D {
                offset,
                target: Vector2::zero(),
                rotation: 0.,
                zoom: scale,
            };
            let mut logical = d.begin_mode2D(camera);
            let command = view::draw(
                &mut logical,
                &fonts,
                &client,
                &scene,
                &texture,
                &pointer,
                cast,
            );
            if view::button(
                &mut logical,
                &fonts,
                &pointer,
                Rectangle::new(25., 806., 170., 32.),
                "< Hub  /  H",
                false,
                true,
            ) {
                return Ok(result);
            }
            command
        };
        if let Some(command) = command {
            match command {
                Command::Snapshot => save = true,
                Command::ResetView => scene.reset_view(),
                Command::TopView => {
                    scene.eye_yaw = 0.;
                    scene.eye_pitch = 1.40;
                    scene.distance = 172.;
                }
                _ => client.command(command),
            }
        }
        if save {
            let dir = snapshots();
            fs::create_dir_all(&dir)?;
            let name = dir.join(format!(
                "orbit-path-{}.png",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs()
            ));
            rl.take_screenshot(thread, name.to_string_lossy().as_ref());
            client.notify("Screenshot saved!");
            save = false;
        }
    }
    Ok(result)
}
