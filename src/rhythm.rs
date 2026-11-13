//! Beat-synchronised paddle duel against Bleep.
use crate::{
    characters::{Cast, Character},
    view::Fonts,
};
use crownfall::reactions::Mood;
use raylib::prelude::*;
pub fn play(
    rl: &mut RaylibHandle,
    t: &RaylibThread,
    cast: &Cast,
) -> Result<Option<i8>, Box<dyn std::error::Error>> {
    let audio = RaylibAudio::init_audio_device().ok();
    let beat_wave = audio.as_ref().and_then(|a| {
        a.new_wave_from_memory(".wav", include_bytes!("../assets/audio/beat.wav"))
            .ok()
    });
    let hit_wave = audio.as_ref().and_then(|a| {
        a.new_wave_from_memory(".wav", include_bytes!("../assets/audio/hit.wav"))
            .ok()
    });
    let beat = audio
        .as_ref()
        .zip(beat_wave.as_ref())
        .and_then(|(a, w)| a.new_sound_from_wave(w).ok());
    let hit = audio
        .as_ref()
        .zip(hit_wave.as_ref())
        .and_then(|(a, w)| a.new_sound_from_wave(w).ok());
    let mut cpu_y = 400f32;
    let fonts = Fonts::load(rl, t);
    let mut time = 0f32;
    let mut emitted = 0;
    let mut notes: Vec<(f32, f32, usize)> = vec![];
    let mut scores = [0, 0];
    let mut combo = 0;
    let mut result = None;
    while !rl.window_should_close() {
        if rl.is_key_pressed(KeyboardKey::KEY_H) || rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            return Ok(result);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            time = 0.;
            emitted = 0;
            notes.clear();
            scores = [0, 0];
            combo = 0;
            result = None;
        }
        let dt = rl.get_frame_time().min(0.05);
        let w = rl.get_screen_width() as f32;
        let h = rl.get_screen_height() as f32;
        let py = rl.get_mouse_position().y.clamp(150., h - 90.);
        if result.is_none() {
            time += dt;
            while emitted < 96 && time >= emitted as f32 * 0.5 {
                let y = 180. + ((emitted * 73 + emitted / 8 * 37) % 400) as f32 * (h - 270.) / 400.;
                notes.push((w - 60., y, emitted));
                emitted += 1;
                if let Some(sound) = &beat {
                    sound.play();
                }
            }
            if let Some(note) = notes.first() {
                cpu_y += (note.1 - cpu_y).clamp(-dt * 245., dt * 245.);
            }
            for n in &mut notes {
                n.0 -= (w - 180.) / 2. * dt;
            }
            notes.retain(|n| {
                if n.0 <= 110. {
                    if (n.1 - py).abs() < 49. {
                        scores[0] += 100 + combo * 5;
                        combo += 1;
                        if let Some(sound) = &hit {
                            sound.play();
                        }
                    } else {
                        combo = 0;
                    }
                    if (n.1 - cpu_y).abs() < 49. {
                        scores[1] += 110;
                    }
                    false
                } else {
                    true
                }
            });
            if emitted == 96 && notes.is_empty() {
                result = Some(if scores[0] > scores[1] {
                    1
                } else if scores[0] == scores[1] {
                    -1
                } else {
                    0
                });
            }
        }
        let mut d = rl.begin_drawing(t);
        d.clear_background(Color::new(9, 12, 35, 255));
        for i in 0..90 {
            let x = ((i * 137) as f32 + time * 12.) % w;
            let y = ((i * 79) % 800) as f32 / 800. * h;
            d.draw_circle(x as i32, y as i32, 1.5, Color::new(120, 145, 190, 150));
        }
        let pulse = 1. - (time * 2.).fract();
        d.draw_circle_lines(
            (w / 2.) as i32,
            (h / 2.) as i32,
            150. + pulse * 100.,
            Color::new(65, 70, 140, 120),
        );
        d.draw_text("ORBIT BEAT / BLEEP CPU", 30, 25, 28, Color::SKYBLUE);
        d.draw_text(
            "Mouse: move paddle | R: restart | H: hub",
            30,
            65,
            18,
            Color::LIGHTGRAY,
        );
        d.draw_text(
            &format!("YOU {}   CPU {}   COMBO {}", scores[0], scores[1], combo),
            30,
            100,
            22,
            Color::GOLD,
        );
        d.draw_rectangle_rounded(
            Rectangle::new(85., py - 48., 22., 96.),
            0.5,
            8,
            Color::SKYBLUE,
        );
        d.draw_rectangle_rounded(
            Rectangle::new(w - 35., cpu_y - 48., 14., 96.),
            0.5,
            8,
            Color::PURPLE,
        );
        for n in &notes {
            d.draw_rectangle(
                (n.0 - 8.) as i32,
                (n.1 - 8.) as i32,
                16,
                16,
                if n.2 % 4 == 0 {
                    Color::GOLD
                } else {
                    Color::MAGENTA
                },
            );
            d.draw_line(
                (n.0 + 9.) as i32,
                n.1 as i32,
                (n.0 + 35.) as i32,
                n.1 as i32,
                Color::PURPLE,
            );
        }
        cast.duel(
            &mut d,
            &fonts,
            Character::Star,
            moods(
                result,
                combo,
                emitted,
                notes.first().is_some_and(|n| n.0 < 160.),
            ),
            time,
        );
        if let Some(r) = result {
            d.draw_text(
                if r == 1 {
                    "YOU WIN!"
                } else if r == 0 {
                    "BLEEP WINS"
                } else {
                    "DRAW"
                },
                (w / 2. - 120.) as i32,
                (h / 2.) as i32,
                40,
                Color::GOLD,
            );
        }
    }
    Ok(result)
}

fn moods(result: Option<i8>, combo: i32, emitted: usize, incoming: bool) -> [Mood; 2] {
    match result {
        Some(1) => [Mood::Happy, Mood::Sad],
        Some(0) => [Mood::Sad, Mood::Happy],
        Some(_) => [Mood::Surprised, Mood::Surprised],
        None if emitted == 0 => [Mood::Idle, Mood::Idle],
        None if combo > 8 => [Mood::Happy, Mood::Nervous],
        None if incoming => [Mood::Thinking, Mood::Surprised],
        None => [Mood::Thinking, Mood::Thinking],
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn robot_reactions_follow_match_state() {
        assert_eq!(moods(None, 9, 20, true)[1], Mood::Nervous);
        assert_eq!(moods(Some(1), 9, 96, false)[1], Mood::Sad);
        assert_eq!(moods(Some(0), 0, 96, false)[1], Mood::Happy);
        assert_eq!(moods(None, 0, 0, false)[1], Mood::Idle);
        assert_eq!(moods(None, 0, 5, true)[1], Mood::Surprised);
        assert_eq!(moods(None, 0, 5, false)[1], Mood::Thinking);
    }
}
