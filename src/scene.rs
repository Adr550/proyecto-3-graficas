//! Reuses the previous project's World, planet geometry, OBJ ship, math, and CPU renderer.
use crate::{
    game::*,
    geometry::{Mesh, box_mesh, ring, sphere},
    math::{M4, V3},
    renderer::{Camera, Renderer},
    shaders::{Kind, Material},
    world::World,
};
use std::f32::consts::PI;
pub const SPACING: f32 = 12.;
pub fn cell_position(cell: usize) -> V3 {
    let (x, y) = xy(cell);
    V3::new((x as f32 - 4.) * SPACING, 0., (y as f32 - 4.) * SPACING)
}
pub fn wall_position(x: u8, y: u8) -> V3 {
    V3::new((x as f32 - 3.5) * SPACING, 1., (y as f32 - 3.5) * SPACING)
}
pub fn biome(cell: usize) -> usize {
    let (x, y) = xy(cell);
    if (x == 1 || x == 7) && (y == 1 || y == 7) {
        2
    } else if (x + y) % 7 == 0 {
        0
    } else if (x * 2 + y) % 9 == 0 {
        3
    } else if (x + y * 2) % 5 == 0 {
        1
    } else {
        4
    }
}
pub struct Preview {
    pub wall: Wall,
    pub valid: bool,
}
pub struct Scene {
    pub world: World,
    base: Mesh,
    rim: Mesh,
    wall: Mesh,
    cap: Mesh,
    pad: Mesh,
    pub renderer: Renderer,
    pub eye_yaw: f32,
    pub eye_pitch: f32,
    pub distance: f32,
    pub ship_positions: [V3; 2],
    pub ship_yaws: [f32; 2],
    pub time: f32,
}
impl Scene {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            world: World::new(),
            base: box_mesh(117., 2.8, 117.),
            rim: box_mesh(120., 1.3, 120.),
            wall: box_mesh(23., 6., 1.2),
            cap: sphere(8, 6, None),
            pad: ring(3.85, 4.5, 32),
            renderer: Renderer::new(width, height),
            eye_yaw: 0.50,
            eye_pitch: 0.96,
            distance: 218.,
            ship_positions: [
                cell_position(76) + V3::new(0., 7.2, 0.),
                cell_position(4) + V3::new(0., 7.2, 0.),
            ],
            ship_yaws: [0., PI],
            time: 0.,
        }
    }
    pub fn reset_view(&mut self) {
        self.eye_yaw = 0.50;
        self.eye_pitch = 0.96;
        self.distance = 218.;
    }
    pub fn camera(&self) -> Camera {
        let target = V3::new(0., 0., 0.);
        Camera {
            eye: V3::new(
                self.eye_yaw.sin() * self.eye_pitch.cos() * self.distance,
                self.eye_pitch.sin() * self.distance,
                self.eye_yaw.cos() * self.eye_pitch.cos() * self.distance,
            ),
            target,
            up: V3::Y,
            fov: 0.68,
        }
    }
    pub fn render(
        &mut self,
        g: &Game,
        legal: &[usize],
        hover: Option<usize>,
        preview: Option<Preview>,
    ) {
        self.renderer.begin(self.camera(), self.time);
        let r = &mut self.renderer;
        // Rounded-console palette on a physical white board, not a replacement 2D grid.
        r.draw(
            &self.rim,
            M4::compose(V3::new(0., -6., 0.), 0., 1., 0.),
            Material::solid([138., 192., 226.]),
            1,
        );
        r.draw(
            &self.base,
            M4::compose(V3::new(0., -4.3, 0.), 0., 1., 0.),
            Material::glowing([245., 250., 255.]),
            2,
        );
        for row in [0, 8] {
            let z = (row as f32 - 4.) * SPACING;
            let color = if row == 0 {
                [98., 197., 248.]
            } else {
                [249., 163., 138.]
            };
            for x in 0..9 {
                let p = cell_position(row * 9 + x);
                r.draw(
                    &self.pad,
                    M4::compose(V3::new(p.x, -2.5, z), 0., 1., 0.),
                    Material::glowing(color),
                    3,
                );
            }
        }
        for cell in 0..CELLS {
            let kind = biome(cell);
            let planet = &self.world.planets[kind];
            let p = cell_position(cell);
            let radius = if kind == 2 { 2.7 } else { 3.15 };
            let rotation = self.time * 0.14 + cell as f32 * 0.7;
            let parent = M4::compose(p, rotation, 1., 0.12);
            let detailed = hover == Some(cell) || g.pawns.contains(&cell);
            let mesh = if detailed {
                &planet.medium
            } else {
                &planet.low
            };
            r.draw(
                mesh,
                parent * M4::compose(V3::ZERO, 0., radius, 0.),
                Material::textured(planet.def.kind),
                (10 + cell) as u16,
            );
            let props_scale = radius / planet.def.radius;
            let props_parent = parent * M4::compose(V3::ZERO, 0., props_scale, 0.);
            if kind == 2 {
                for part in &planet.children {
                    r.draw(
                        &part.mesh,
                        props_parent * part.model,
                        part.material,
                        (10 + cell) as u16,
                    );
                }
            } else if kind == 1 || kind == 0 || kind == 3 {
                for part in planet.children.iter().take(if kind == 1 { 4 } else { 2 }) {
                    r.draw(
                        &part.mesh,
                        props_parent * part.model,
                        part.material,
                        (10 + cell) as u16,
                    );
                }
            } else if detailed {
                for part in planet.children.iter().rev().take(5) {
                    r.draw(
                        &part.mesh,
                        props_parent * part.model,
                        part.material,
                        (10 + cell) as u16,
                    );
                }
            }
        }
        // Thin 3D lanes connect planet centers; occupied and selected spaces stay visible.
        for cell in 0..CELLS {
            for d in [1, 2] {
                if let Some(next) = neighbor(cell, d)
                    && !g.blocked(cell, next)
                {
                    let mut a = cell_position(cell);
                    let mut b = cell_position(next);
                    a.y = -2.7;
                    b.y = -2.7;
                    r.line(a, b, [129., 183., 210.], 0.48);
                }
            }
        }
        if g.turn == 0 && g.winner.is_none() {
            for &cell in legal {
                r.draw(
                    &self.pad,
                    M4::compose(cell_position(cell) + V3::new(0., -1.5, 0.), 0., 1.03, 0.),
                    Material::glowing([59., 185., 237.]),
                    100,
                );
            }
        }
        for wall in &g.walls {
            draw_wall(
                r,
                &self.wall,
                &self.cap,
                *wall,
                if wall.owner == 0 {
                    [66., 174., 238.]
                } else {
                    [248., 143., 122.]
                },
                1.,
            );
        }
        if let Some(preview) = preview {
            draw_wall(
                r,
                &self.wall,
                &self.cap,
                preview.wall,
                if preview.valid {
                    [107., 222., 174.]
                } else {
                    [250., 150., 157.]
                },
                0.88,
            );
        }
        for player in 0..2 {
            let color = if player == 0 {
                [103., 199., 253.]
            } else {
                [255., 155., 137.]
            };
            let bob = (self.time * 2. + player as f32 * PI).sin() * 0.28;
            let pos = self.ship_positions[player] + V3::new(0., bob, 0.);
            r.draw(
                &self.world.ship,
                M4::compose(pos, self.ship_yaws[player], 0.85, 0.),
                Material {
                    color,
                    ..Material::textured(Kind::Ship)
                },
                (200 + player) as u16,
            );
            r.glow(pos - V3::new(0., 2., 0.), 2.3, color, 0.15);
        }
        r.finish();
    }
    pub fn pick_cell(&self, x: f32, y: f32) -> Option<usize> {
        let index = (y as usize).min(self.renderer.height - 1) * self.renderer.width
            + (x as usize).min(self.renderer.width - 1);
        let id = self.renderer.ids[index];
        if (10..91).contains(&id) {
            return Some((id - 10) as usize);
        }
        let mut best = None;
        let mut distance = 20f32.powi(2);
        for cell in 0..CELLS {
            let p = self
                .renderer
                .project(cell_position(cell) + V3::new(0., 2.2, 0.));
            if p.w <= 0. {
                continue;
            }
            let d = (p.x - x).powi(2) + (p.y - y).powi(2);
            if d < distance {
                distance = d;
                best = Some(cell);
            }
        }
        best
    }
    pub fn pick_wall(&self, x: f32, y: f32) -> Option<(u8, u8)> {
        let mut best = None;
        let mut distance = 26f32.powi(2);
        for wy in 0..8 {
            for wx in 0..8 {
                let p = self
                    .renderer
                    .project(wall_position(wx, wy) + V3::new(0., 2., 0.));
                let d = (p.x - x).powi(2) + (p.y - y).powi(2);
                if d < distance {
                    distance = d;
                    best = Some((wx, wy));
                }
            }
        }
        best
    }
}
fn draw_wall(r: &mut Renderer, beam: &Mesh, cap: &Mesh, w: Wall, color: [f32; 3], scale: f32) {
    let p = wall_position(w.x, w.y) + V3::new(0., 1.5, 0.);
    let yaw = if w.axis == Axis::Horizontal {
        0.
    } else {
        PI * 0.5
    };
    r.draw(
        beam,
        M4::compose(p, yaw, scale, 0.),
        Material::solid(color),
        300,
    );
    for sign in [-1., 1.] {
        let delta = if w.axis == Axis::Horizontal {
            V3::new(sign * 10.8, 3., 0.)
        } else {
            V3::new(0., 3., sign * 10.8)
        };
        r.draw(
            cap,
            M4::compose(p + delta, 0., 0.9, 0.),
            Material::solid([242., 250., 255.]),
            300,
        );
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn planets_and_original_ship_are_used() {
        let s = Scene::new(160, 120);
        assert_eq!(s.world.planets.len(), 5);
        assert!(s.world.ship.vertices.len() > 30);
        assert_eq!(cell_position(0), V3::new(-48., 0., -48.));
        assert_eq!(cell_position(80), V3::new(48., 0., 48.));
    }
    #[test]
    fn render_contains_both_ship_models_and_planets() {
        let mut s = Scene::new(400, 300);
        let g = Game::new();
        s.render(&g, &g.moves(0), None, None);
        assert!(s.renderer.stats.triangles > 20000);
        assert!(s.renderer.ids.contains(&200));
        assert!(s.renderer.ids.contains(&201));
        assert!(
            s.renderer
                .ids
                .iter()
                .filter(|&&id| (10..91).contains(&id))
                .count()
                > 500
        );
    }
}
