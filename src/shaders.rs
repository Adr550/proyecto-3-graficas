//! Clean, colorful toy-like surfaces for the console-sports visual direction.
use crate::math::{V3, fbm, lerp, smooth};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Kind {
    Paper,
    Terra,
    Gas,
    Lava,
    Fractal,
    Forest,
    Sun,
    Ring,
    Ship,
}
#[derive(Clone, Copy)]
pub struct Material {
    pub color: [f32; 3],
    pub kind: Kind,
    pub double_sided: bool,
    pub emissive: bool,
    pub normal_map: bool,
    pub opacity: f32,
    pub tip_color: Option<[f32; 3]>,
    pub gloss: f32,
}
impl Material {
    pub fn solid(color: [f32; 3]) -> Self {
        Self {
            color,
            kind: Kind::Paper,
            double_sided: false,
            emissive: false,
            normal_map: true,
            opacity: 1.,
            tip_color: None,
            gloss: 0.25,
        }
    }
    pub fn textured(kind: Kind) -> Self {
        Self {
            color: [255.; 3],
            kind,
            double_sided: matches!(kind, Kind::Ring),
            emissive: matches!(kind, Kind::Sun),
            normal_map: !matches!(kind, Kind::Gas | Kind::Ring | Kind::Sun),
            opacity: 1.,
            tip_color: None,
            gloss: 0.25,
        }
    }
    pub fn color_at(self, v: f32) -> [f32; 3] {
        if let Some(tip) = self.tip_color {
            let t = ((v - 0.48) / 0.52).clamp(0., 1.);
            let t = t * t * (3. - 2. * t);
            std::array::from_fn(|i| self.color[i] + (tip[i] - self.color[i]) * t)
        } else {
            self.color
        }
    }
    pub fn glowing(color: [f32; 3]) -> Self {
        Self {
            emissive: true,
            ..Self::solid(color)
        }
    }
}
pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub color: Vec<[f32; 3]>,
    pub normals: Vec<V3>,
    pub emission: Vec<f32>,
}
#[derive(Clone, Copy)]
pub struct Sample {
    pub color: [f32; 3],
    pub normal: V3,
    pub emission: f32,
}
fn mix_color(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    std::array::from_fn(|i| lerp(a[i], b[i], t))
}
impl Texture {
    pub fn generate(kind: Kind, width: usize, height: usize) -> Self {
        let mut colors = vec![[0.; 3]; width * height];
        let mut heights = vec![0.; width * height];
        let mut emission = vec![0.; width * height];
        for y in 0..height {
            for x in 0..width {
                let (u, v) = (x as f32 / width as f32, y as f32 / height as f32);
                let warp = (fbm(u * 7. + 4., v * 8. + 9., 3.) - 0.5) * 0.018;
                let (u1, v1) = (u + warp, v + warp * 0.6);
                let (a, p) = (u1 * std::f32::consts::TAU, v1 * std::f32::consts::PI);
                let n3 = V3::new(p.sin() * a.cos(), p.cos(), p.sin() * a.sin());
                let n = fbm(n3.x * 4. + 12., n3.y * 4. + 7., n3.z * 4. + 21.);
                let bump;
                let mut emit = 0.;
                let color = match kind {
                    Kind::Terra | Kind::Forest => {
                        let shift = if kind == Kind::Forest { 40. } else { 31. };
                        let land = fbm(n3.x * 2.7 + shift, n3.y * 2.7 + 10., n3.z * 2.7 + 50.);
                        let ocean = mix_color([47., 166., 223.], [100., 205., 235.], n);
                        let green = if kind == Kind::Forest {
                            [76., 190., 100.]
                        } else {
                            [80., 183., 120.]
                        };
                        let coast = smooth(0.435, 0.485, land);
                        let interior = smooth(0.475, 0.515, land);
                        let sand = mix_color(ocean, [241., 224., 160.], coast);
                        let mut c =
                            mix_color(sand, mix_color(green, [146., 220., 111.], n), interior);
                        // Capillary pigment pools along the softened shoreline.
                        let pool = (smooth(0.46, 0.48, land) - smooth(0.48, 0.51, land)) * 0.12;
                        for cc in &mut c {
                            *cc *= 1. - pool;
                        }
                        if n3.y.abs() > 0.92 {
                            c = mix_color(c, [228., 225., 196.], smooth(0.92, 0.995, n3.y.abs()));
                        }
                        bump = land * interior;
                        c
                    }
                    Kind::Gas => {
                        let flow = (u1 * 19.).sin() * 0.36 + (u1 * 7. + v1 * 17.).sin() * 0.7;
                        let band = (v1 * 43. + flow).sin() * 0.5 + 0.5;
                        let mut c = mix_color(
                            [246., 163., 104.],
                            [255., 231., 159.],
                            smooth(0.15, 0.9, band),
                        );
                        let storm = ((u1 - 0.67) * 18.).powi(2) + ((v1 - 0.58) * 13.).powi(2);
                        if storm < 2. {
                            c = mix_color([235., 124., 113.], c, smooth(0.1, 1.9, storm));
                        }
                        bump = 0.;
                        c
                    }
                    Kind::Lava => {
                        let crack = ((n3.x * 14. + (n3.z * 17.).sin() + n3.y * 7.).sin()
                            * (n3.y * 13. + (n3.x * 9.).sin()).cos())
                        .abs();
                        let hot = 1. - smooth(0.012, 0.095, crack);
                        let bleed = 1. - smooth(0.045, 0.18, crack);
                        let rock = mix_color([105., 112., 150.], [142., 139., 177.], n);
                        emit = hot * 0.85;
                        bump = n * 0.7;
                        mix_color(
                            mix_color(rock, [248., 130., 81.], bleed * 0.55),
                            [255., 191., 74.],
                            hot,
                        )
                    }
                    Kind::Fractal => {
                        let (mut zx, mut zy, mut k) = (0., 0., 0);
                        let (cx, cy) = ((u1 - 0.56) * 3.5, (v1 - 0.5) * 2.7);
                        while k < 32 && zx * zx + zy * zy < 4. {
                            let q = zx * zx - zy * zy + cx;
                            zy = 2. * zx * zy + cy;
                            zx = q;
                            k += 1;
                        }
                        bump = k as f32 / 42.;
                        let t = if k == 32 {
                            0.
                        } else {
                            (k as f32 * 0.13).sin() * 0.5 + 0.5
                        };
                        mix_color([137., 122., 222.], [203., 196., 255.], t)
                    }
                    Kind::Sun => {
                        bump = n;
                        mix_color([255., 188., 62.], [255., 241., 126.], smooth(0.15, 0.8, n))
                    }
                    Kind::Ring => {
                        let bands = (v1 * 58.).sin() * 0.4 + (v1 * 23.).sin() * 0.22;
                        let density = (0.5 + bands).clamp(0., 1.);
                        bump = 0.;
                        mix_color([226., 180., 126.], [255., 235., 194.], density)
                    }
                    Kind::Ship => {
                        let xx = (u * 8.).fract();
                        let yy = (v * 4.).fract();
                        let edge = 1. - smooth(0.02, 0.055, xx.min(yy));
                        bump = 1. - edge;
                        let c = mix_color([246., 250., 255.], [167., 197., 218.], edge * 0.7);
                        if xx > 0.68 && xx < 0.8 {
                            mix_color(c, [190., 206., 217.], 0.8)
                        } else {
                            c
                        }
                    }
                    Kind::Paper => {
                        bump = n * 0.3;
                        mix_color([255., 255., 255.], [255., 255., 255.], n)
                    }
                };
                let at = y * width + x;
                colors[at] = color;
                heights[at] = bump;
                emission[at] = emit;
            }
        }
        let mut normals = vec![V3::ZERO; width * height];
        for y in 0..height {
            for x in 0..width {
                let dx = (heights[y * width + (x + 1) % width]
                    - heights[y * width + (x + width - 1) % width])
                    * 0.45;
                let dy = (heights[((y + 1) % height) * width + x]
                    - heights[((y + height - 1) % height) * width + x])
                    * 0.45;
                normals[y * width + x] = V3::new(-dx, -dy, 1.).normalized();
            }
        }
        Self {
            width,
            height,
            color: colors,
            normals,
            emission,
        }
    }
    #[inline]
    pub fn sample(&self, u: f32, v: f32) -> Sample {
        let fx = u.rem_euclid(1.) * self.width as f32;
        let fy = v.clamp(0., 0.999999) * self.height as f32;
        // rem_euclid can round a tiny negative f32 to exactly 1.0.
        // Wrap the integer texel as well, so seams never index past the map.
        let (x, y) = (
            (fx.floor() as usize) % self.width,
            (fy.floor() as usize).min(self.height - 1),
        );
        let (dx, dy) = (fx.fract(), fy.fract());
        let x1 = (x + 1) % self.width;
        let y1 = (y + 1).min(self.height - 1);
        let samples = [
            y * self.width + x,
            y * self.width + x1,
            y1 * self.width + x,
            y1 * self.width + x1,
        ];
        let weights = [
            (1. - dx) * (1. - dy),
            dx * (1. - dy),
            (1. - dx) * dy,
            dx * dy,
        ];
        let mut color = [0.; 3];
        for i in 0..4 {
            for (c, channel) in color.iter_mut().enumerate() {
                *channel += self.color[samples[i]][c] * weights[i];
            }
        }
        Sample {
            color,
            normal: self.normals[samples[0]],
            emission: self.emission[samples[0]],
        }
    }
}
pub struct Textures {
    pub maps: Vec<Texture>,
}
impl Textures {
    pub fn new() -> Self {
        Self {
            maps: [
                Kind::Paper,
                Kind::Terra,
                Kind::Gas,
                Kind::Lava,
                Kind::Fractal,
                Kind::Forest,
                Kind::Sun,
                Kind::Ring,
                Kind::Ship,
            ]
            .into_iter()
            .map(|k| Texture::generate(k, 512, 256))
            .collect(),
        }
    }
    pub fn get(&self, k: Kind) -> &Texture {
        &self.maps[k as usize]
    }
}
impl Default for Textures {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn textures_are_deterministic_and_nonuniform() {
        let a = Texture::generate(Kind::Gas, 32, 16);
        let b = Texture::generate(Kind::Gas, 32, 16);
        assert_eq!(a.color, b.color);
        assert_ne!(a.color[0], a.color[200]);
        for n in a.normals {
            assert!((n.length() - 1.).abs() < 1e-4);
        }
    }
    #[test]
    fn uv_wrap_and_bilinear() {
        let t = Texture::generate(Kind::Terra, 32, 16);
        for (a, b) in t
            .sample(0.4, 0.5)
            .color
            .into_iter()
            .zip(t.sample(1.4, 0.5).color)
        {
            assert!((a - b).abs() < 0.001);
        }
        assert!(t.sample(-0.01, 0.5).color.iter().all(|c| c.is_finite()));
        for u in [-f32::EPSILON * 0.01, 0.0, 1.0, 1.000001] {
            assert!(t.sample(u, 1.0).color.iter().all(|c| c.is_finite()));
        }
    }
}
