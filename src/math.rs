//! Our vector, matrix, noise, and seeded-random math. No raylib camera or 3D math.
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl V3 {
    pub const ZERO: Self = Self::new(0., 0., 0.);
    pub const Y: Self = Self::new(0., 1., 0.);
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub fn dot(self, b: Self) -> f32 {
        self.x * b.x + self.y * b.y + self.z * b.z
    }
    pub fn cross(self, b: Self) -> Self {
        Self::new(
            self.y * b.z - self.z * b.y,
            self.z * b.x - self.x * b.z,
            self.x * b.y - self.y * b.x,
        )
    }
    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }
    pub fn normalized(self) -> Self {
        let l = self.length();
        if l > 1e-8 { self / l } else { Self::ZERO }
    }
    pub fn lerp(self, b: Self, t: f32) -> Self {
        self + (b - self) * t
    }
}
impl Add for V3 {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        Self::new(self.x + b.x, self.y + b.y, self.z + b.z)
    }
}
impl Sub for V3 {
    type Output = Self;
    fn sub(self, b: Self) -> Self {
        Self::new(self.x - b.x, self.y - b.y, self.z - b.z)
    }
}
impl Mul<f32> for V3 {
    type Output = Self;
    fn mul(self, b: f32) -> Self {
        Self::new(self.x * b, self.y * b, self.z * b)
    }
}
impl Div<f32> for V3 {
    type Output = Self;
    fn div(self, b: f32) -> Self {
        self * (1. / b)
    }
}
impl Neg for V3 {
    type Output = Self;
    fn neg(self) -> Self {
        self * -1.
    }
}
impl AddAssign for V3 {
    fn add_assign(&mut self, b: Self) {
        *self = *self + b
    }
}
#[derive(Clone, Copy, Debug)]
pub struct M4(pub [f32; 16]);
impl M4 {
    pub fn identity() -> Self {
        Self([
            1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
        ])
    }
    pub fn compose(p: V3, yaw: f32, s: f32, tilt: f32) -> Self {
        let (c, q) = (yaw.cos(), yaw.sin());
        let (ct, st) = (tilt.cos(), tilt.sin());
        Self([
            c * s,
            0.,
            -q * s,
            0.,
            q * st * s,
            ct * s,
            c * st * s,
            0.,
            q * ct * s,
            -st * s,
            c * ct * s,
            0.,
            p.x,
            p.y,
            p.z,
            1.,
        ])
    }
    pub fn basis(x: V3, y: V3, z: V3, p: V3, s: f32) -> Self {
        Self([
            x.x * s,
            x.y * s,
            x.z * s,
            0.,
            y.x * s,
            y.y * s,
            y.z * s,
            0.,
            z.x * s,
            z.y * s,
            z.z * s,
            0.,
            p.x,
            p.y,
            p.z,
            1.,
        ])
    }
    pub fn surface(n: V3, r: f32, s: f32) -> Self {
        let y = n.normalized();
        let axis = if y.y.abs() > 0.95 {
            V3::new(1., 0., 0.)
        } else {
            V3::Y
        };
        let x = axis.cross(y).normalized();
        Self::basis(x, y, x.cross(y), y * r, s)
    }
    pub fn point(self, v: V3) -> V3 {
        let p = self.transform([v.x, v.y, v.z, 1.]);
        V3::new(p[0], p[1], p[2])
    }
    pub fn direction(self, v: V3) -> V3 {
        let p = self.transform([v.x, v.y, v.z, 0.]);
        V3::new(p[0], p[1], p[2])
    }
    pub fn transform(self, p: [f32; 4]) -> [f32; 4] {
        let m = self.0;
        std::array::from_fn(|r| m[r] * p[0] + m[4 + r] * p[1] + m[8 + r] * p[2] + m[12 + r] * p[3])
    }
    pub fn position(self) -> V3 {
        V3::new(self.0[12], self.0[13], self.0[14])
    }
    pub fn scale(self) -> f32 {
        V3::new(self.0[0], self.0[1], self.0[2]).length()
    }
    pub fn normal_matrix(self) -> Self {
        let x = V3::new(self.0[0], self.0[1], self.0[2]);
        let y = V3::new(self.0[4], self.0[5], self.0[6]);
        let z = V3::new(self.0[8], self.0[9], self.0[10]);
        let determinant = x.dot(y.cross(z));
        let inverse = if determinant.abs() > 1e-9 {
            1. / determinant
        } else {
            1.
        };
        Self::basis(
            y.cross(z) * inverse,
            z.cross(x) * inverse,
            x.cross(y) * inverse,
            V3::ZERO,
            1.,
        )
    }
    pub fn bounding_scale(self) -> f32 {
        [
            V3::new(self.0[0], self.0[1], self.0[2]).length(),
            V3::new(self.0[4], self.0[5], self.0[6]).length(),
            V3::new(self.0[8], self.0[9], self.0[10]).length(),
        ]
        .into_iter()
        .fold(0., f32::max)
    }
    pub fn look_at(eye: V3, target: V3, up: V3) -> Self {
        let z = (eye - target).normalized();
        let x = up.cross(z).normalized();
        let y = z.cross(x);
        Self([
            x.x,
            y.x,
            z.x,
            0.,
            x.y,
            y.y,
            z.y,
            0.,
            x.z,
            y.z,
            z.z,
            0.,
            -x.dot(eye),
            -y.dot(eye),
            -z.dot(eye),
            1.,
        ])
    }
    pub fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = 1. / (fov * 0.5).tan();
        let nf = 1. / (near - far);
        Self([
            f / aspect,
            0.,
            0.,
            0.,
            0.,
            f,
            0.,
            0.,
            0.,
            0.,
            (far + near) * nf,
            -1.,
            0.,
            0.,
            2. * far * near * nf,
            0.,
        ])
    }
}
impl Mul for M4 {
    type Output = Self;
    fn mul(self, b: Self) -> Self {
        let mut o = [0.; 16];
        for c in 0..4 {
            for r in 0..4 {
                o[c * 4 + r] = (0..4).map(|k| self.0[k * 4 + r] * b.0[c * 4 + k]).sum();
            }
        }
        Self(o)
    }
}
#[derive(Clone)]
pub struct Rng(u32);
impl Rng {
    pub fn new(seed: u32) -> Self {
        Self(seed)
    }
    pub fn next_f32(&mut self) -> f32 {
        self.0 = self.0.wrapping_add(0x6d2b79f5);
        let mut t = self.0;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        ((t ^ (t >> 14)) as f64 / 4294967296.) as f32
    }
}
pub fn hash(x: i32, y: i32, z: i32) -> f32 {
    let mut n = (x as u32)
        .wrapping_mul(374761393)
        .wrapping_add((y as u32).wrapping_mul(668265263))
        .wrapping_add((z as u32).wrapping_mul(2147483647));
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    ((n ^ (n >> 16)) as f64 / 4294967295.) as f32
}
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
pub fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}
pub fn noise(x: f32, y: f32, z: f32) -> f32 {
    let (ix, iy, iz) = (x.floor() as i32, y.floor() as i32, z.floor() as i32);
    let f = |t: f32| t * t * (3. - 2. * t);
    let (u, v, w) = (f(x - x.floor()), f(y - y.floor()), f(z - z.floor()));
    let plane = |dz| {
        lerp(
            lerp(hash(ix, iy, iz + dz), hash(ix + 1, iy, iz + dz), u),
            lerp(hash(ix, iy + 1, iz + dz), hash(ix + 1, iy + 1, iz + dz), u),
            v,
        )
    };
    lerp(plane(0), plane(1), w)
}
pub fn fbm(x: f32, y: f32, z: f32) -> f32 {
    noise(x, y, z) * 0.57
        + noise(x * 2.03, y * 2.03, z * 2.03) * 0.28
        + noise(x * 4.07, y * 4.07, z * 4.07) * 0.15
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matrix_composition() {
        let m = M4::compose(V3::new(2., 3., 4.), 0., 2., 0.)
            * M4::compose(V3::new(1., 0., 0.), 0., 1., 0.);
        assert_eq!(m.point(V3::ZERO), V3::new(4., 3., 4.));
    }
    #[test]
    fn seeded_repeatable() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_f32(), b.next_f32());
        }
    }
    #[test]
    fn frame_radial() {
        let n = V3::new(0.3, 0.8, 0.2).normalized();
        let m = M4::surface(n, 20., 1.);
        assert!((m.direction(V3::Y) - n).length() < 1e-5);
        assert!((m.position() - n * 20.).length() < 1e-5);
    }
}
