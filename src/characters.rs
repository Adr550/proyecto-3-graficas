//! Original illustrated cast, with Fer's updated six-expression astronaut sheet.
#![allow(clippy::too_many_arguments)]
use crate::view::{self, Fonts};
use crownfall::reactions::Mood;
use raylib::prelude::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Character {
    Fer,
    Pig,
    Cat,
    Star,
    Ember,
    Prism,
}
impl Character {
    pub const ALL: [Self; 6] = [
        Self::Fer,
        Self::Pig,
        Self::Cat,
        Self::Star,
        Self::Ember,
        Self::Prism,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Fer => "Fer / You",
            Self::Pig => "Captain Truffle",
            Self::Cat => "Mochi",
            Self::Star => "Bleep / Space Robot",
            Self::Ember => "Ember",
            Self::Prism => "Lumen",
        }
    }
    pub fn planet(i: usize) -> Self {
        [Self::Ember, Self::Star, Self::Pig, Self::Prism, Self::Cat][i.min(4)]
    }
    pub fn color(self) -> Color {
        match self {
            Self::Fer => Color::new(50, 171, 202, 255),
            Self::Pig => Color::new(237, 151, 178, 255),
            Self::Cat => Color::new(146, 196, 70, 255),
            Self::Star => Color::new(91, 179, 214, 255),
            Self::Ember => Color::new(237, 146, 85, 255),
            Self::Prism => Color::new(165, 148, 219, 255),
        }
    }
}
const ORIGINAL: &[u8] = include_bytes!("../assets/characters/orbit-cast-atlas-v2.png");
const BLEEP: &[u8] = include_bytes!("../assets/characters/bleep-expressions-v1.png");
const FER: &[u8] = include_bytes!("../assets/characters/fer-expressions-v1.png");
fn sprite_rect(c: Character, m: Mood, width: i32, height: i32) -> Rectangle {
    let (columns, rows, column, row) = if matches!(c, Character::Fer | Character::Star) {
        (2, 3, m as usize % 2, m as usize / 2)
    } else {
        (6, 6, m as usize, c as usize)
    };
    let cw = width as f32 / columns as f32;
    let ch = height as f32 / rows as f32;
    Rectangle::new(column as f32 * cw, row as f32 * ch, cw, ch)
}
fn face_image(image: &Image, c: Character, m: Mood) -> Image {
    let source = sprite_rect(c, m, image.width(), image.height());
    let colors = image.get_image_data();
    let mut face = Image::gen_image_color(128, 128, Color::BLANK);
    for y in 0..128 {
        for x in 0..128 {
            let edge = (((x as f32 - 63.5).powi(2) + (y as f32 - 63.5).powi(2)).sqrt() - 62.)
                .clamp(0., 1.);
            if edge >= 1. {
                continue;
            }
            let sx = (source.x + source.width * (0.08 + x as f32 / 128. * 0.84)) as usize;
            let sy = (source.y + source.height * (y as f32 / 128. * 0.84)) as usize;
            let mut pixel = colors[sy * image.width() as usize + sx];
            pixel.a = (pixel.a as f32 * (1. - edge)) as u8;
            face.draw_pixel(x, y, pixel);
        }
    }
    face
}
pub struct Cast {
    original: Texture2D,
    fer: Texture2D,
    bleep: Texture2D,
    faces: Vec<Texture2D>,
}
impl Cast {
    pub fn load(
        rl: &mut RaylibHandle,
        t: &RaylibThread,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let original = Image::load_image_from_mem(".png", ORIGINAL)?;
        let fer = Image::load_image_from_mem(".png", FER)?;
        let bleep = Image::load_image_from_mem(".png", BLEEP)?;
        let mut faces = Vec::with_capacity(36);
        for c in Character::ALL {
            for m in [
                Mood::Idle,
                Mood::Thinking,
                Mood::Nervous,
                Mood::Happy,
                Mood::Sad,
                Mood::Surprised,
            ] {
                let image = if c == Character::Fer {
                    &fer
                } else if c == Character::Star {
                    &bleep
                } else {
                    &original
                };
                let face = face_image(image, c, m);
                let texture = rl.load_texture_from_image(t, &face)?;
                texture.set_texture_filter(t, TextureFilter::TEXTURE_FILTER_BILINEAR);
                faces.push(texture);
            }
        }
        let original = rl.load_texture_from_image(t, &original)?;
        let fer = rl.load_texture_from_image(t, &fer)?;
        let bleep = rl.load_texture_from_image(t, &bleep)?;
        for texture in [&original, &fer, &bleep] {
            texture.set_texture_filter(t, TextureFilter::TEXTURE_FILTER_BILINEAR);
        }
        Ok(Self {
            original,
            fer,
            bleep,
            faces,
        })
    }
    pub fn body(&self, d: &mut impl RaylibDraw, c: Character, m: Mood, r: Rectangle, time: f32) {
        let atlas = if c == Character::Fer {
            &self.fer
        } else if c == Character::Star {
            &self.bleep
        } else {
            &self.original
        };
        let source = sprite_rect(c, m, atlas.width, atlas.height);
        let wobble = if m == Mood::Nervous {
            (time * 30.).sin() * 1.7
        } else {
            0.
        };
        let bounce = if m == Mood::Happy {
            (time * 7.).sin().abs() * r.height * 0.015
        } else {
            (time * 2.4).sin() * r.height * 0.01
        };
        d.draw_texture_pro(
            atlas,
            source,
            Rectangle::new(r.x + wobble, r.y + bounce, r.width, r.height),
            Vector2::zero(),
            0.,
            Color::WHITE,
        );
    }
    pub fn portrait(
        &self,
        d: &mut impl RaylibDraw,
        c: Character,
        m: Mood,
        center: Vector2,
        r: f32,
        time: f32,
    ) {
        let x = center.x
            + if m == Mood::Nervous {
                (time * 31.).sin() * 1.2
            } else {
                0.
            };
        d.draw_circle(
            x as i32,
            (center.y + 3.) as i32,
            r + 3.,
            Color::new(78, 125, 150, 25),
        );
        d.draw_circle(x as i32, center.y as i32, r + 2., c.color());
        d.draw_circle(
            x as i32,
            center.y as i32,
            r - 1.,
            Color::new(248, 253, 255, 255),
        );
        d.draw_texture_pro(
            &self.faces[c as usize * 6 + m as usize],
            Rectangle::new(0., 0., 128., 128.),
            Rectangle::new(x - r + 1., center.y - r + 1., r * 2. - 2., r * 2. - 2.),
            Vector2::zero(),
            0.,
            Color::WHITE,
        );
    }
    pub fn duel(
        &self,
        d: &mut impl RaylibDraw,
        f: &Fonts,
        host: Character,
        moods: [Mood; 2],
        time: f32,
    ) {
        for (i, c) in [Character::Fer, host].iter().enumerate() {
            let x = 785. + i as f32 * 233.;
            view::panel(d, Rectangle::new(x, 20., 222., 84.));
            self.portrait(d, *c, moods[i], Vector2::new(x + 39., 60.), 31., time);
            view::text(d, f, c.name(), x + 79., 37., 16., c.color(), true);
            view::text(d, f, moods[i].label(), x + 79., 64., 13., view::DARK, false);
        }
    }
}
pub fn export_portraits(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let original = Image::load_image_from_mem(".png", ORIGINAL)?;
    let fer = Image::load_image_from_mem(".png", FER)?;
    let bleep = Image::load_image_from_mem(".png", BLEEP)?;
    let mut sheet = Image::gen_image_color(900, 900, Color::new(225, 240, 247, 255));
    for (row, c) in Character::ALL.into_iter().enumerate() {
        for (col, m) in [
            Mood::Idle,
            Mood::Thinking,
            Mood::Nervous,
            Mood::Happy,
            Mood::Sad,
            Mood::Surprised,
        ]
        .into_iter()
        .enumerate()
        {
            let face = face_image(
                if c == Character::Fer {
                    &fer
                } else if c == Character::Star {
                    &bleep
                } else {
                    &original
                },
                c,
                m,
            );
            sheet.draw_circle(col as i32 * 150 + 75, row as i32 * 150 + 75, 66, c.color());
            sheet.draw_circle(
                col as i32 * 150 + 75,
                row as i32 * 150 + 75,
                63,
                Color::new(249, 253, 255, 255),
            );
            sheet.draw(
                &face,
                Rectangle::new(0., 0., 128., 128.),
                Rectangle::new(col as f32 * 150. + 11., row as f32 * 150. + 11., 128., 128.),
                Color::WHITE,
            );
        }
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    crownfall::png::write_png(path, 900, 900, &sheet.get_image_data_u8(false))?;
    Ok(())
}
pub fn export_art(dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(dir)?;
    let original = Image::load_image_from_mem(".png", ORIGINAL)?;
    let fer = Image::load_image_from_mem(".png", FER)?;
    let bleep = Image::load_image_from_mem(".png", BLEEP)?;
    let mut lineup = Image::gen_image_color(1200, 800, Color::new(228, 241, 248, 255));
    for (i, c) in Character::ALL.into_iter().enumerate() {
        let image = if c == Character::Fer {
            &fer
        } else if c == Character::Star {
            &bleep
        } else {
            &original
        };
        lineup.draw(
            image,
            sprite_rect(c, Mood::Idle, image.width(), image.height()),
            Rectangle::new((i % 3) as f32 * 400., (i / 3) as f32 * 400., 400., 400.),
            Color::WHITE,
        );
    }
    crownfall::png::write_png(
        dir.join("illustrated-cast.png"),
        1200,
        800,
        &lineup.get_image_data_u8(false),
    )?;
    let mut fer_preview = Image::gen_image_color(512, 512, Color::new(234, 243, 249, 255));
    fer_preview.draw(
        &fer,
        sprite_rect(Character::Fer, Mood::Idle, fer.width(), fer.height()),
        Rectangle::new(0., 0., 512., 512.),
        Color::WHITE,
    );
    crownfall::png::write_png(
        dir.join("fer-illustration.png"),
        512,
        512,
        &fer_preview.get_image_data_u8(false),
    )?;
    export_portraits(&dir.join("character-expressions.png"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bleep_sheet_supports_six_transparent_poses_and_portraits() {
        let image = Image::load_image_from_mem(".png", BLEEP).unwrap();
        assert_eq!((image.width(), image.height()), (1024, 1536));
        let data = image.get_image_data();
        for m in [
            Mood::Idle,
            Mood::Thinking,
            Mood::Nervous,
            Mood::Happy,
            Mood::Sad,
            Mood::Surprised,
        ] {
            let r = sprite_rect(Character::Star, m, image.width(), image.height());
            let mut empty = 0;
            let mut solid = 0;
            for y in r.y as i32..(r.y + r.height) as i32 {
                for x in r.x as i32..(r.x + r.width) as i32 {
                    let a = data[(y * image.width() + x) as usize].a;
                    empty += usize::from(a == 0);
                    solid += usize::from(a > 128);
                }
            }
            assert!(empty > 10000 && solid > 10000);
            assert_eq!(face_image(&image, Character::Star, m).get_color(0, 0).a, 0);
        }
    }
    #[test]
    fn original_hosts_use_the_exact_original_cells() {
        for c in [
            Character::Pig,
            Character::Cat,
            Character::Ember,
            Character::Prism,
        ] {
            for m in [
                Mood::Idle,
                Mood::Thinking,
                Mood::Nervous,
                Mood::Happy,
                Mood::Sad,
                Mood::Surprised,
            ] {
                let r = sprite_rect(c, m, 1254, 1254);
                assert_eq!(r.x, m as usize as f32 * 209.);
                assert_eq!(r.y, c as usize as f32 * 209.);
            }
        }
        assert_eq!(Character::planet(1), Character::Star);
    }
    #[test]
    fn fer_has_all_six_nonempty_transparent_expression_cells() {
        let image = Image::load_image_from_mem(".png", FER).unwrap();
        assert_eq!(image.width() % 2, 0);
        assert_eq!(image.height() % 3, 0);
        let data = image.get_image_data();
        assert_eq!(data.iter().map(|p| p.a).min(), Some(0));
        for m in [
            Mood::Idle,
            Mood::Thinking,
            Mood::Nervous,
            Mood::Happy,
            Mood::Sad,
            Mood::Surprised,
        ] {
            let r = sprite_rect(Character::Fer, m, image.width(), image.height());
            let width = image.width();
            let count = (r.y as i32..(r.y + r.height) as i32)
                .flat_map(|y| {
                    (r.x as i32..(r.x + r.width) as i32).map(move |x| (y * width + x) as usize)
                })
                .filter(|&i| data[i].a > 100)
                .count();
            assert!(count as f32 > r.width * r.height * 0.1);
            let face = face_image(&image, Character::Fer, m);
            assert_eq!(face.get_color(0, 0).a, 0);
        }
    }
    #[test]
    fn restored_cast_atlas_decodes_with_transparency() {
        let image = Image::load_image_from_mem(".png", ORIGINAL).unwrap();
        assert_eq!((image.width(), image.height()), (1254, 1254));
        assert_eq!(image.get_image_data().iter().map(|p| p.a).min(), Some(0));
    }
}
