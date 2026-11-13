//! Five time-based orbits, procedural surface attachments, and reusable modeled props.
use crate::geometry::{Mesh, box_mesh, cone, ring, ship_mesh, sphere};
use crate::math::{M4, Rng, V3, fbm, noise};
use crate::renderer::Renderer;
use crate::shaders::{Kind, Material};
use std::{f32::consts::TAU, rc::Rc};
#[derive(Clone, Copy)]
pub struct PlanetDef {
    pub name: &'static str,
    pub subtitle: &'static str,
    pub description: &'static str,
    pub feature: &'static str,
    pub kind: Kind,
    pub color: [u8; 3],
    pub radius: f32,
    pub orbit: f32,
    pub period: f32,
    pub spin: f32,
    pub phase: f32,
    pub inclination: f32,
    pub seed: u32,
}
pub const DEFINITIONS: [PlanetDef; 5] = [
    PlanetDef {
        name: "Cinder",
        subtitle: "THE VOLCANIC WORLD",
        description: "Smoky violet stone, warm vermilion rivers, and open craters. A little world that never quite cools down.",
        feature: "Active volcanoes",
        kind: Kind::Lava,
        color: [221, 145, 118],
        radius: 13.,
        orbit: 62.,
        period: 72.,
        spin: 31.,
        phase: 2.45,
        inclination: 0.08,
        seed: 17,
    },
    PlanetDef {
        name: "Relic",
        subtitle: "THE FORGOTTEN CIVILIZATION",
        description: "Sea-glass oceans wrap an abandoned civilization. Weathered arches and quiet towers remember its story.",
        feature: "Ancient ruins",
        kind: Kind::Terra,
        color: [158, 192, 151],
        radius: 17.,
        orbit: 108.,
        period: 110.,
        spin: 53.,
        phase: 4.7,
        inclination: 0.03,
        seed: 42,
    },
    PlanetDef {
        name: "Aurelia",
        subtitle: "THE RINGED GAS GIANT",
        description: "Apricot cloud washes and a rose-colored storm. Two great rings drift like brushstrokes across the sky.",
        feature: "Ice and dust rings",
        kind: Kind::Gas,
        color: [225, 188, 143],
        radius: 23.,
        orbit: 169.,
        period: 160.,
        spin: 41.,
        phase: 0.2,
        inclination: 0.09,
        seed: 73,
    },
    PlanetDef {
        name: "Prism",
        subtitle: "THE FRACTAL WORLD",
        description: "A mathematical watercolor. Recursive crystal gardens grow from lavender terrain and endless Mandelbrot patterns.",
        feature: "Recursive crystals",
        kind: Kind::Fractal,
        color: [172, 163, 211],
        radius: 16.,
        orbit: 235.,
        period: 220.,
        spin: 67.,
        phase: 3.25,
        inclination: 0.06,
        seed: 91,
    },
    PlanetDef {
        name: "Bloom",
        subtitle: "THE GARDEN WORLD",
        description: "One enormous tree, a tiny living planet. Sage hills, blush canopies, and turquoise seas fill its soft horizon.",
        feature: "The great canopy",
        kind: Kind::Forest,
        color: [149, 190, 160],
        radius: 20.,
        orbit: 307.,
        period: 290.,
        spin: 79.,
        phase: 5.35,
        inclination: 0.04,
        seed: 126,
    },
];
#[derive(Clone)]
pub struct Object {
    pub mesh: Rc<Mesh>,
    pub material: Material,
    pub model: M4,
    pub detail: bool,
}
pub struct Planet {
    pub def: PlanetDef,
    pub pos: V3,
    pub rotation: f32,
    pub high: Rc<Mesh>,
    pub medium: Rc<Mesh>,
    pub low: Rc<Mesh>,
    pub children: Vec<Object>,
}
pub struct World {
    pub planets: Vec<Planet>,
    pub sun: Mesh,
    pub ship: Mesh,
    pub elapsed: f64,
    pub orbital_angles: [f64; 5],
    pub orbit_speed: [f32; 5],
}
struct Models {
    block: Rc<Mesh>,
    lintel: Rc<Mesh>,
    column: Rc<Mesh>,
    mountain: Rc<Mesh>,
    crater: Rc<Mesh>,
    trunk: Rc<Mesh>,
    leaf: Rc<Mesh>,
    crystal: Rc<Mesh>,
    step: Rc<Mesh>,
    cube: Rc<Mesh>,
}
impl Models {
    fn new() -> Self {
        Self {
            block: Rc::new(box_mesh(2.3, 5., 2.3)),
            lintel: Rc::new(box_mesh(7., 1.5, 2.4)),
            column: Rc::new(cone(0.8, 4.3, 10, 0.7)),
            mountain: Rc::new(cone(3.8, 5.4, 18, 1.25)),
            crater: Rc::new(ring(0., 1.18, 24)),
            trunk: Rc::new(cone(1.2, 8., 10, 0.65)),
            leaf: Rc::new(sphere(18, 12, None)),
            crystal: Rc::new(cone(1., 3.1, 4, 0.)),
            step: Rc::new(box_mesh(5.5, 0.9, 5.5)),
            cube: Rc::new(box_mesh(1., 1., 1.)),
        }
    }
}
fn object(mesh: &Rc<Mesh>, color: [f32; 3], p: V3, s: f32, yaw: f32) -> Object {
    Object {
        mesh: mesh.clone(),
        material: Material::solid(color),
        model: M4::compose(p, yaw, s, 0.),
        detail: false,
    }
}
const STONE: [f32; 3] = [191., 197., 173.];
const CREAM: [f32; 3] = [231., 216., 180.];
const MOSS: [f32; 3] = [107., 153., 125.];
fn ruins(m: &Models, variant: usize) -> Vec<Object> {
    match variant {
        0 => vec![
            object(&m.column, CREAM, V3::new(-2., 0., 0.), 1., 0.),
            object(&m.column, CREAM, V3::new(2., 0., 0.), 1., 0.),
            object(&m.lintel, STONE, V3::new(0., 4.4, 0.), 1., 0.),
            object(&m.step, MOSS, V3::ZERO, 1., 0.),
        ],
        1 => vec![
            object(&m.block, STONE, V3::new(0., 2., 0.), 1., 0.),
            object(&m.block, CREAM, V3::new(0.5, 6.2, 0.), 0.7, 0.18),
            object(&m.step, MOSS, V3::ZERO, 1., 0.),
            object(&m.cube, MOSS, V3::new(1.3, 4.4, 0.), 1.5, 0.),
        ],
        _ => vec![
            object(&m.step, CREAM, V3::ZERO, 1., 0.),
            object(&m.step, STONE, V3::new(0., 0.8, 0.), 0.8, 0.),
            object(&m.crystal, MOSS, V3::new(0., 1.2, 0.), 1.8, 0.),
            object(&m.block, STONE, V3::new(4., 0., 1.), 0.38, 0.8),
        ],
    }
}
fn tree(m: &Models, big: bool, rand: &mut Rng) -> Vec<Object> {
    let mut out = vec![object(&m.trunk, [141., 106., 85.], V3::ZERO, 1., 0.)];
    for i in 0..if big { 9 } else { 3 } {
        let a = i as f32 * 2.4;
        let r = if big { 3.8 } else { 1.8 };
        let color = if i % 3 == 0 {
            [219., 167., 181.]
        } else {
            [135., 180., 126.]
        };
        out.push(object(
            &m.leaf,
            color,
            V3::new(a.cos() * r, 6. + rand.next_f32() * 3., a.sin() * r),
            if big { 3.1 } else { 2.2 },
            0.,
        ));
    }
    out
}
fn fractal(m: &Models, depth: usize, size: f32, p: V3, out: &mut Vec<Object>) {
    let color = if depth % 2 == 1 {
        [186., 174., 219.]
    } else {
        [139., 156., 194.]
    };
    out.push(object(&m.cube, color, p, size, 0.));
    if depth > 0 {
        for d in [
            V3::new(1., 1., 0.),
            V3::new(-1., 1., 0.),
            V3::new(0., 1., 1.),
            V3::new(0., 1., -1.),
            V3::new(0., 2., 0.),
        ] {
            fractal(m, depth - 1, size * 0.47, p + d * (size * 0.85), out);
        }
    }
}
pub fn terrain(kind: Kind, p: V3) -> f32 {
    match kind {
        Kind::Lava => 1. + (fbm(p.x * 5. + 9., p.y * 5., p.z * 5.) - 0.45) * 0.19,
        Kind::Fractal => 1. + ((p.x * 9.).sin() * (p.y * 9.).sin() * (p.z * 9.).sin()).abs() * 0.09,
        Kind::Terra | Kind::Forest => 1. + (fbm(p.x * 4. + 20., p.y * 4., p.z * 4.) - 0.5) * 0.06,
        _ => 1.,
    }
}
impl Planet {
    fn build(def: PlanetDef, m: &Models) -> Self {
        let deform = |p| terrain(def.kind, p);
        let high = Rc::new(sphere(64, 36, Some(&deform)));
        let medium = Rc::new(sphere(32, 20, Some(&deform)));
        let low = Rc::new(sphere(18, 12, Some(&deform)));
        let mut children = Vec::new();
        let mut rand = Rng::new(def.seed);
        let mut points: Vec<V3> = Vec::new();
        let count = match def.kind {
            Kind::Gas => 0,
            Kind::Terra => 17,
            _ => 12,
        };
        for _ in 0..500 {
            if points.len() >= count {
                break;
            }
            let y = rand.next_f32() * 1.8 - 0.9;
            let a = rand.next_f32() * TAU;
            let r = (1. - y * y).sqrt();
            let n = V3::new(r * a.cos(), y, r * a.sin());
            if noise(n.x * 3. + def.seed as f32, n.y * 3., n.z * 3.) < 0.40
                || points.iter().any(|p| (*p - n).length() < 0.35)
            {
                continue;
            }
            points.push(n);
        }
        for (i, n) in points.iter().enumerate() {
            let s = match def.kind {
                Kind::Forest => 0.6,
                Kind::Fractal => 0.9,
                _ => 0.7 + rand.next_f32() * 0.45,
            };
            let frame = M4::surface(*n, def.radius * terrain(def.kind, *n), s);
            let parts = match def.kind {
                Kind::Terra => ruins(m, i % 3),
                Kind::Forest => tree(m, false, &mut rand),
                Kind::Fractal => {
                    let mut v = vec![];
                    fractal(m, 2, 1.6, V3::ZERO, &mut v);
                    v
                }
                Kind::Lava => {
                    let mut crater =
                        object(&m.crater, [252., 179., 107.], V3::new(0., 5.42, 0.), 1., 0.);
                    crater.material = Material::glowing([248., 168., 98.]);
                    vec![
                        object(&m.mountain, [94., 84., 109.], V3::ZERO, 1., 0.),
                        crater,
                    ]
                }
                _ => vec![],
            };
            for mut part in parts {
                part.model = frame * part.model;
                part.detail = i % 2 == 1;
                children.push(part);
            }
        }
        if def.kind == Kind::Forest {
            let frame = M4::surface(V3::new(0.15, 1., -0.2).normalized(), def.radius * 0.98, 1.6);
            for mut part in tree(m, true, &mut rand) {
                part.model = frame * part.model;
                children.push(part);
            }
        }
        if def.kind == Kind::Gas {
            for (a, b) in [(1.35, 1.72), (1.82, 2.12)] {
                children.push(Object {
                    mesh: Rc::new(ring(a, b, 128)),
                    material: Material::textured(Kind::Ring),
                    model: M4::compose(V3::ZERO, 0., def.radius, 0.32),
                    detail: false,
                });
            }
        }
        Self {
            def,
            pos: V3::ZERO,
            rotation: 0.,
            high,
            medium,
            low,
            children,
        }
    }
    pub fn collision_radius(&self) -> f32 {
        self.def.radius
            * if self.def.kind == Kind::Lava {
                1.10
            } else {
                1.04
            }
            + 3.
    }
}
impl World {
    pub fn new() -> Self {
        let models = Models::new();
        let mut w = Self {
            planets: DEFINITIONS
                .into_iter()
                .map(|d| Planet::build(d, &models))
                .collect(),
            sun: sphere(48, 28, None),
            ship: ship_mesh().expect("The embedded ship model must be valid"),
            elapsed: 0.,
            orbital_angles: DEFINITIONS.map(|d| d.phase as f64),
            orbit_speed: [1.; 5],
        };
        w.update_positions();
        w
    }
    pub fn update(&mut self, dt: f32) {
        self.elapsed += dt as f64;
        for (i, p) in self.planets.iter_mut().enumerate() {
            self.orbital_angles[i] += dt as f64 / p.def.period as f64
                * std::f64::consts::TAU
                * self.orbit_speed[i] as f64;
            p.rotation = (self.elapsed / p.def.spin as f64 * std::f64::consts::TAU)
                .rem_euclid(std::f64::consts::TAU) as f32;
        }
        self.update_positions();
    }
    pub fn set_time(&mut self, time: f32) {
        self.elapsed = time as f64;
        for (i, p) in self.planets.iter_mut().enumerate() {
            self.orbital_angles[i] = (p.def.phase + time / p.def.period * TAU) as f64;
            p.rotation = (time / p.def.spin * TAU).rem_euclid(TAU);
        }
        self.update_positions();
    }
    fn update_positions(&mut self) {
        for (i, p) in self.planets.iter_mut().enumerate() {
            let a = self.orbital_angles[i];
            p.pos = V3::new(
                a.cos() as f32 * p.def.orbit,
                a.sin() as f32 * p.def.orbit * p.def.inclination,
                a.sin() as f32 * p.def.orbit,
            );
        }
    }
    pub fn nearest(&self, position: V3) -> usize {
        self.planets
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                ((position - a.pos).length() - a.def.radius)
                    .total_cmp(&((position - b.pos).length() - b.def.radius))
            })
            .map(|(i, _)| i)
            .unwrap_or(0)
    }
    pub fn collide(&self, mut position: V3, mut velocity: V3) -> (V3, V3, bool) {
        let mut collided = false;
        let bodies = std::iter::once((V3::ZERO, 31.))
            .chain(self.planets.iter().map(|p| (p.pos, p.collision_radius())));
        for (center, radius) in bodies {
            let delta = position - center;
            let distance = delta.length();
            if distance < radius {
                let n = if distance > 1e-5 {
                    delta / distance
                } else {
                    V3::Y
                };
                position = center + n * radius;
                let inward = velocity.dot(n);
                if inward < 0. {
                    velocity = velocity - n * inward;
                }
                collided = true;
            }
        }
        (position, velocity, collided)
    }
    pub fn draw(
        &self,
        r: &mut Renderer,
        show_paths: bool,
        selected: Option<usize>,
        ship_model: Option<M4>,
    ) {
        r.glow(V3::ZERO, 78., [215., 137., 89.], 0.5);
        r.glow(V3::ZERO, 45., [243., 181., 114.], 0.42);
        r.draw(
            &self.sun,
            M4::compose(V3::ZERO, self.elapsed as f32 * 0.03, 27., 0.),
            Material::textured(Kind::Sun),
            1,
        );
        for (i, p) in self.planets.iter().enumerate() {
            let ratio = (r.camera.eye - p.pos).length() / p.def.radius;
            let g = if !r.options.lod {
                &p.high
            } else if ratio > 15. {
                r.stats.lower_lod += 1;
                &p.low
            } else if ratio > 7. {
                r.stats.lower_lod += 1;
                &p.medium
            } else {
                &p.high
            };
            let parent = M4::compose(p.pos, p.rotation, 1., 0.12);
            r.draw(
                g,
                parent * M4::compose(V3::ZERO, 0., p.def.radius, 0.),
                Material::textured(p.def.kind),
                (i + 2) as u16,
            );
            if ratio < 32. || !r.options.lod {
                for part in &p.children {
                    if r.options.lod && ratio > 17. && part.detail {
                        continue;
                    }
                    r.draw(
                        &part.mesh,
                        parent * part.model,
                        part.material,
                        (i + 2) as u16,
                    );
                }
            }
        }
        if show_paths {
            for (i, p) in self.planets.iter().enumerate() {
                let mut previous = None;
                for j in 0..=160 {
                    let a = j as f32 / 160. * TAU;
                    let position = V3::new(
                        a.cos() * p.def.orbit,
                        a.sin() * p.def.orbit * p.def.inclination,
                        a.sin() * p.def.orbit,
                    );
                    if let Some(old) = previous {
                        r.line(
                            old,
                            position,
                            [136., 160., 178.],
                            if selected == Some(i) { 0.35 } else { 0.14 },
                        );
                    }
                    previous = Some(position);
                }
            }
        }
        if let Some(m) = ship_model {
            r.draw(&self.ship, m, Material::textured(Kind::Ship), 10);
        }
    }
}
impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn procedural_placement_repeatable() {
        let m = Models::new();
        let a = Planet::build(DEFINITIONS[1], &m);
        let b = Planet::build(DEFINITIONS[1], &m);
        assert!(a.children.len() > 20);
        assert_eq!(a.children.len(), b.children.len());
        for (x, y) in a.children.iter().zip(b.children) {
            assert_eq!(x.model.0, y.model.0);
        }
        assert!(a.high.indices.len() > a.low.indices.len());
        assert!((a.high.radius - 1.).abs() > 0.001);
    }
    #[test]
    fn elapsed_orbits_independent_of_steps() {
        let mut a = World::new();
        let mut b = World::new();
        for _ in 0..600 {
            a.update(1. / 60.);
        }
        for _ in 0..100 {
            b.update(0.1);
        }
        for (x, y) in a.planets.iter().zip(b.planets) {
            assert!((x.pos - y.pos).length() < 0.02);
        }
        assert!(a.planets[0].def.period < a.planets[4].def.period);
    }
    #[test]
    fn collision_removes_inward_motion() {
        let w = World::new();
        let p = &w.planets[1];
        let pos = p.pos + V3::new(1., 0., 0.);
        let (result, velocity, hit) = w.collide(pos, V3::new(-20., 3., 0.));
        assert!(hit);
        assert!((result - p.pos).length() >= p.collision_radius() - 0.0001);
        assert!(velocity.x >= 0.);
    }
}
