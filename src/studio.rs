//! Preview the restored 2D cast and all six reactive expressions.
use crate::{
    characters::{Cast, Character},
    view::*,
};
use crownfall::reactions::Mood;
use raylib::prelude::*;
pub fn run(
    rl: &mut RaylibHandle,
    t: &RaylibThread,
    cast: &Cast,
) -> Result<(), Box<dyn std::error::Error>> {
    let f = Fonts::load(rl, t);
    let mut selected = 0;
    let mut expression = 0;
    let mut time = 0.;
    rl.set_window_title(t, "Orbit Club / Character Gallery");
    while !rl.window_should_close() {
        if rl.is_key_pressed(KeyboardKey::KEY_H) || rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            break;
        }
        time += rl.get_frame_time().clamp(0., 0.1);
        let scale =
            (rl.get_screen_width() as f32 / WIDTH).min(rl.get_screen_height() as f32 / HEIGHT);
        let offset = Vector2::new(
            (rl.get_screen_width() as f32 - WIDTH * scale) / 2.,
            (rl.get_screen_height() as f32 - HEIGHT * scale) / 2.,
        );
        let mouse = rl.get_mouse_position();
        let p = Pointer {
            point: Vector2::new((mouse.x - offset.x) / scale, (mouse.y - offset.y) / scale),
            clicked: rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT),
        };
        if rl.is_key_pressed(KeyboardKey::KEY_TAB) {
            selected = (selected + 1) % 6;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            expression = (expression + 1) % 6;
        }
        for (i, k) in [
            KeyboardKey::KEY_ONE,
            KeyboardKey::KEY_TWO,
            KeyboardKey::KEY_THREE,
            KeyboardKey::KEY_FOUR,
            KeyboardKey::KEY_FIVE,
            KeyboardKey::KEY_SIX,
        ]
        .iter()
        .enumerate()
        {
            if rl.is_key_pressed(*k) {
                selected = i;
            }
        }
        let mood = [
            Mood::Idle,
            Mood::Thinking,
            Mood::Nervous,
            Mood::Happy,
            Mood::Sad,
            Mood::Surprised,
        ][expression];
        let mut leave = false;
        {
            let mut d = rl.begin_drawing(t);
            d.clear_background(EDGE);
            let mut d = d.begin_mode2D(Camera2D {
                offset,
                target: Vector2::zero(),
                rotation: 0.,
                zoom: scale,
            });
            d.clear_background(Color::new(228, 244, 253, 255));
            text(&mut d, &f, "CHARACTER GALLERY", 32., 28., 32., BLUE, true);
            text(
                &mut d,
                &f,
                "1-6 or Tab: choose a character / Space: next expression",
                34.,
                79.,
                17.,
                GREY,
                false,
            );
            panel(&mut d, rect(25., 130., 830., 680.));
            cast.body(
                &mut d,
                Character::ALL[selected],
                mood,
                rect(140., 150., 600., 600.),
                time,
            );
            cast.portrait(
                &mut d,
                Character::ALL[selected],
                mood,
                Vector2::new(765., 210.),
                60.,
                time,
            );
            panel(&mut d, rect(880., 130., 372., 680.));
            text(
                &mut d,
                &f,
                Character::ALL[selected].name(),
                906.,
                152.,
                24.,
                DARK,
                true,
            );
            for (i, c) in Character::ALL.iter().enumerate() {
                if button(
                    &mut d,
                    &f,
                    &p,
                    rect(906., 203. + i as f32 * 48., 320., 37.),
                    &format!("{}  {}", i + 1, c.name()),
                    selected == i,
                    true,
                ) {
                    selected = i;
                }
            }
            text(&mut d, &f, "EXPRESSION", 906., 510., 16., BLUE, true);
            for (i, label) in [
                "Idle",
                "Thinking",
                "Nervous",
                "Celebrate",
                "Sad",
                "Surprised",
            ]
            .iter()
            .enumerate()
            {
                if button(
                    &mut d,
                    &f,
                    &p,
                    rect(
                        906. + (i % 2) as f32 * 165.,
                        545. + (i / 2) as f32 * 49.,
                        154.,
                        38.,
                    ),
                    label,
                    expression == i,
                    true,
                ) {
                    expression = i;
                }
            }
            if button(
                &mut d,
                &f,
                &p,
                rect(906., 755., 320., 37.),
                "< Return / H",
                true,
                true,
            ) {
                leave = true;
            }
        }
        if leave {
            break;
        }
    }
    Ok(())
}
