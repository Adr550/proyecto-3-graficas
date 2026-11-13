//! Custom CPU transform, clip, rasterization, fragment, normal-map, and Z-buffer stages.
use crate::geometry::Mesh;
use crate::math::{M4, Rng, V3, hash};
use crate::shaders::{Kind, Material, Textures};
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub eye: V3,
    pub target: V3,
    pub up: V3,
    pub fov: f32,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: V3::new(355., 285., 392.),
            target: V3::ZERO,
            up: V3::Y,
            fov: 0.8,
        }
    }
}
#[derive(Default, Clone, Copy, Debug)]
pub struct Stats {
    pub objects: usize,
    pub culled: usize,
    pub triangles: usize,
    pub rasterized: usize,
    pub fragments: usize,
    pub lower_lod: usize,
}
#[derive(Clone, Copy)]
pub struct Options {
    pub culling: bool,
    pub lod: bool,
    pub normal_maps: bool,
    pub ink: bool,
    pub watercolor: f32,
    pub studio: bool,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            culling: true,
            lod: true,
            normal_maps: false,
            ink: false,
            watercolor: 0.,
            studio: false,
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct ClipVertex(pub [f32; 10]);
impl ClipVertex {
    fn distance(self, plane: usize) -> f32 {
        let v = self.0;
        match plane {
            0 => v[3] + v[0],
            1 => v[3] - v[0],
            2 => v[3] + v[1],
            3 => v[3] - v[1],
            4 => v[3] + v[2],
            _ => v[3] - v[2],
        }
    }
    fn lerp(self, b: Self, t: f32) -> Self {
        Self(std::array::from_fn(|i| {
            self.0[i] + (b.0[i] - self.0[i]) * t
        }))
    }
}
pub fn clip_polygon(input: &[ClipVertex]) -> Vec<ClipVertex> {
    let mut poly = input.to_vec();
    for plane in 0..6 {
        if poly.is_empty() {
            break;
        }
        let mut out = Vec::with_capacity(poly.len() + 2);
        let mut previous = *poly.last().unwrap();
        let mut pd = previous.distance(plane);
        for &current in &poly {
            let cd = current.distance(plane);
            if (pd >= 0.) != (cd >= 0.) {
                out.push(previous.lerp(current, pd / (pd - cd)));
            }
            if cd >= 0. {
                out.push(current);
            }
            previous = current;
            pd = cd;
        }
        poly = out;
    }
    poly
}
#[derive(Clone, Copy)]
pub struct Projected {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
pub struct Renderer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
    pub depth: Vec<f32>,
    pub ids: Vec<u16>,
    sky: Vec<u8>,
    grain: Vec<f32>,
    pub textures: Textures,
    pub options: Options,
    pub stats: Stats,
    pub camera: Camera,
    view: M4,
    projection: M4,
    vp: M4,
    time: f32,
}
impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut r = Self {
            width: 0,
            height: 0,
            pixels: vec![],
            depth: vec![],
            ids: vec![],
            sky: vec![],
            grain: vec![],
            textures: Textures::new(),
            options: Options::default(),
            stats: Stats::default(),
            camera: Camera::default(),
            view: M4::identity(),
            projection: M4::identity(),
            vp: M4::identity(),
            time: 0.,
        };
        r.resize(width, height);
        r
    }
    pub fn resize(&mut self, w: usize, h: usize) {
        if (w, h) == (self.width, self.height) {
            return;
        }
        self.width = w;
        self.height = h;
        self.pixels = vec![255; w * h * 4];
        self.sky = vec![255; w * h * 4];
        self.depth = vec![f32::INFINITY; w * h];
        self.ids = vec![0; w * h];
        self.grain = vec![0.; w * h];
        let mut rand = Rng::new(2036);
        for y in 0..h {
            for x in 0..w {
                let (nx, ny) = (x as f32 / w as f32, y as f32 / h as f32);
                let neb = (-((nx - 0.62).powi(2) / 0.16 + (ny - 0.46).powi(2) / 0.35)).exp();
                let fine = hash(x as i32, y as i32, 27) - 0.5;
                let fiber = hash((x / 2) as i32, (y / 5) as i32, 51) - 0.5;
                self.grain[y * w + x] = fine * 0.72 + fiber * 0.28;
                let c = [
                    194. + ny * 37. + neb * 6.,
                    224. + ny * 21. + neb * 3.,
                    245. + ny * 8.,
                ];
                let i = (y * w + x) * 4;
                for (k, ch) in c.iter().enumerate() {
                    self.sky[i + k] = *ch as u8;
                }
            }
        }
        for _ in 0..w * h / 480 {
            let x = (rand.next_f32() * w as f32) as usize;
            let y = (rand.next_f32() * h as f32) as usize;
            if x < w && y < h {
                let i = (y * w + x) * 4;
                let b = rand.next_f32() * 12. + 240.;
                for k in 0..3 {
                    self.sky[i + k] = (b * (0.98 + k as f32 * 0.01)) as u8;
                }
            }
        }
    }
    pub fn begin(&mut self, camera: Camera, time: f32) {
        self.camera = camera;
        self.time = time;
        self.view = M4::look_at(camera.eye, camera.target, camera.up);
        self.projection = M4::perspective(
            camera.fov,
            self.width as f32 / self.height as f32,
            0.35,
            2500.,
        );
        self.vp = self.projection * self.view;
        self.pixels.copy_from_slice(&self.sky);
        self.depth.fill(f32::INFINITY);
        self.ids.fill(0);
        self.stats = Stats::default();
    }
    pub fn project(&self, p: V3) -> Projected {
        let q = self.vp.transform([p.x, p.y, p.z, 1.]);
        Projected {
            x: (q[0] / q[3] * 0.5 + 0.5) * self.width as f32,
            y: (0.5 - q[1] / q[3] * 0.5) * self.height as f32,
            z: q[2] / q[3],
            w: q[3],
        }
    }
    pub fn glow(&mut self, p: V3, radius: f32, color: [f32; 3], strength: f32) {
        let p = self.project(p);
        if p.w <= 0. {
            return;
        }
        let r = radius / p.w.max(1.) * self.height as f32 / (self.camera.fov * 0.5).tan() * 0.5;
        if r < 1. {
            return;
        }
        let (x0, x1, y0, y1) = (
            (p.x - r).floor().max(0.) as usize,
            (p.x + r).ceil().min((self.width - 1) as f32).max(0.) as usize,
            (p.y - r).floor().max(0.) as usize,
            (p.y + r).ceil().min((self.height - 1) as f32).max(0.) as usize,
        );
        if x0 > x1 || y0 > y1 {
            return;
        }
        for y in y0..=y1 {
            for x in x0..=x1 {
                let d = ((x as f32 - p.x).powi(2) + (y as f32 - p.y).powi(2)) / (r * r);
                if d > 1. {
                    continue;
                }
                let a = (1. - d).powi(3) * strength;
                let at = (y * self.width + x) * 4;
                for (k, c) in color.iter().enumerate() {
                    self.pixels[at + k] = (self.pixels[at + k] as f32 + c * a).min(255.) as u8;
                }
            }
        }
    }
    pub fn visible(&self, p: V3, radius: f32) -> bool {
        let q = self.vp.transform([p.x, p.y, p.z, 1.]);
        let z = -self.view.point(p).z;
        if z + radius < 0.35 || z - radius > 2500. {
            return false;
        }
        q[0].abs() < q[3] + radius * (self.projection.0[0].powi(2) + 1.).sqrt()
            && q[1].abs() < q[3] + radius * (self.projection.0[5].powi(2) + 1.).sqrt()
    }
    pub fn draw(&mut self, mesh: &Mesh, model: M4, material: Material, id: u16) {
        self.stats.objects += 1;
        if self.options.culling
            && !self.visible(model.position(), mesh.radius * model.bounding_scale())
        {
            self.stats.culled += 1;
            return;
        }
        let mvp = self.vp * model;
        let normal_matrix = model.normal_matrix();
        let mut transformed = Vec::with_capacity(mesh.vertices.len());
        for vertex in &mesh.vertices {
            let p = vertex.p;
            let q = mvp.transform([p.x, p.y, p.z, 1.]);
            let world = model.point(p);
            let n = normal_matrix.direction(vertex.n).normalized();
            let mut t = V3::Y.cross(vertex.n).normalized();
            if t.length() < 0.1 {
                t = V3::new(1., 0., 0.);
            }
            let tangent = model.direction(t).normalized();
            let bitangent = n.cross(tangent).normalized();
            let light = if self.options.studio {
                (V3::new(-4., 7., 9.) - world * 0.15).normalized()
            } else {
                (-world).normalized()
            };
            let view = (self.camera.eye - world).normalized();
            transformed.push(ClipVertex([
                q[0],
                q[1],
                q[2],
                q[3],
                vertex.uv[0],
                vertex.uv[1],
                light.dot(tangent),
                light.dot(bitangent),
                light.dot(n),
                (1. - n.dot(view)).max(0.),
            ]));
        }
        self.stats.triangles += mesh.indices.len() / 3;
        for face in mesh.indices.chunks_exact(3) {
            let triangle = [
                transformed[face[0]],
                transformed[face[1]],
                transformed[face[2]],
            ];
            let mut rejected = false;
            let mut inside = true;
            for plane in 0..6 {
                let ds = triangle.map(|v| v.distance(plane));
                if ds.iter().all(|d| *d < 0.) {
                    rejected = true;
                    break;
                }
                if ds.iter().any(|d| *d < 0.) {
                    inside = false;
                }
            }
            if rejected {
                continue;
            }
            if inside {
                self.triangle(triangle, material, id);
            } else {
                let clipped = clip_polygon(&triangle);
                for i in 1..clipped.len().saturating_sub(1) {
                    self.triangle([clipped[0], clipped[i], clipped[i + 1]], material, id);
                }
            }
        }
    }
    fn triangle(&mut self, vertices: [ClipVertex; 3], mat: Material, id: u16) {
        let [a, b, c] = vertices.map(|v| v.0);
        let (ia, ib, ic) = (1. / a[3], 1. / b[3], 1. / c[3]);
        let (w, h) = (self.width as f32, self.height as f32);
        let (ax, ay, bx, by, cx, cy) = (
            (a[0] * ia * 0.5 + 0.5) * w,
            (0.5 - a[1] * ia * 0.5) * h,
            (b[0] * ib * 0.5 + 0.5) * w,
            (0.5 - b[1] * ib * 0.5) * h,
            (c[0] * ic * 0.5 + 0.5) * w,
            (0.5 - c[1] * ic * 0.5) * h,
        );
        let area = (bx - ax) * (cy - ay) - (by - ay) * (cx - ax);
        if area.abs() < 0.035 || (!mat.double_sided && area >= 0.) {
            return;
        }
        let (x0, x1, y0, y1) = (
            ax.min(bx).min(cx).floor().max(0.) as usize,
            ax.max(bx).max(cx).floor().min(w - 1.).max(0.) as usize,
            ay.min(by).min(cy).floor().max(0.) as usize,
            ay.max(by).max(cy).floor().min(h - 1.).max(0.) as usize,
        );
        if x0 > x1 || y0 > y1 {
            return;
        }
        self.stats.rasterized += 1;
        let inv = 1. / area;
        let dwa = (by - cy) * inv;
        let dwb = (cy - ay) * inv;
        let (az, bz, cz) = (a[2] * ia, b[2] * ib, c[2] * ic);
        let aa: [f32; 10] = std::array::from_fn(|i| a[i] * ia);
        let bb: [f32; 10] = std::array::from_fn(|i| b[i] * ib);
        let cc: [f32; 10] = std::array::from_fn(|i| c[i] * ic);
        let texture = self.textures.get(mat.kind);
        for y in y0..=y1 {
            let px = x0 as f32 + 0.5;
            let py = y as f32 + 0.5;
            let mut wa = ((bx - px) * (cy - py) - (by - py) * (cx - px)) * inv;
            let mut wb = ((cx - px) * (ay - py) - (cy - py) * (ax - px)) * inv;
            for x in x0..=x1 {
                let wc = 1. - wa - wb;
                if wa >= -0.0001 && wb >= -0.0001 && wc >= -0.0001 {
                    let z = wa * az + wb * bz + wc * cz;
                    let at = y * self.width + x;
                    if z < self.depth[at] {
                        let reciprocal = 1. / (wa * ia + wb * ib + wc * ic);
                        let attr = |i: usize| (wa * aa[i] + wb * bb[i] + wc * cc[i]) * reciprocal;
                        let (mut u, mut v) = (attr(4), attr(5));
                        if mat.kind == Kind::Sun {
                            u += (v * 18. + self.time * 0.6).sin() * 0.012 + self.time * 0.007;
                            v += (u * 22. - self.time * 0.8).sin() * 0.008;
                        }
                        let sample = texture.sample(u, v);
                        let mut light = attr(8);
                        if self.options.normal_maps && mat.normal_map {
                            light = attr(6) * sample.normal.x
                                + attr(7) * sample.normal.y
                                + light * sample.normal.z;
                        }
                        if mat.double_sided {
                            light = light.abs();
                        }
                        let diffuse = light.max(0.);
                        let shade = 0.84 + diffuse * 0.15;
                        let band = if light > 0.72 {
                            0.96
                        } else if light > 0.28 {
                            0.82
                        } else if light > -0.05 {
                            0.67
                        } else {
                            0.52
                        };
                        let lighting = if self.options.studio {
                            0.69 + diffuse * 0.27
                        } else {
                            shade * 0.75 + band * 0.25
                        };
                        let emission = if mat.emissive {
                            1.
                        } else if mat.kind == Kind::Lava {
                            sample.emission * (0.90 + 0.10 * (self.time * 1.8 + u * 13.).sin())
                        } else {
                            0.
                        };
                        let lighting = emission + (1. - emission) * lighting;
                        let rim = attr(9);
                        let edge = (rim - 0.72).max(0.) * 0.06;
                        let grain = self.grain[at] * 5.4 * self.options.watercolor;
                        let at4 = at * 4;
                        let alpha = mat.opacity.clamp(0., 1.);
                        let destination_alpha = self.pixels[at4 + 3] as f32 / 255.;
                        let output_alpha = alpha + destination_alpha * (1. - alpha);
                        let tint = mat.color_at(v);
                        for (k, &tint_channel) in tint.iter().enumerate() {
                            let pigment = sample.color[k] * tint_channel / 255.;
                            let paper_lift = if k == 0 {
                                5.
                            } else if k == 1 {
                                4.
                            } else {
                                2.
                            };
                            let color = pigment * lighting
                                + edge * 40.
                                + grain
                                + paper_lift * self.options.watercolor;
                            let specular = if self.options.studio {
                                (1. - rim).max(0.).powf(16.)
                                    * diffuse.powf(3.)
                                    * (8. + mat.gloss * 42.)
                            } else {
                                0.
                            };
                            let old = self.pixels[at4 + k] as f32;
                            self.pixels[at4 + k] = if output_alpha > 0.0001 {
                                (((color + specular) * alpha
                                    + old * destination_alpha * (1. - alpha))
                                    / output_alpha)
                                    .clamp(0., 255.) as u8
                            } else {
                                0
                            };
                        }
                        self.pixels[at4 + 3] = (output_alpha * 255.).round() as u8;
                        if alpha >= 0.999 {
                            self.depth[at] = z;
                        }
                        self.ids[at] = id;
                        self.stats.fragments += 1;
                    }
                }
                wa += dwa;
                wb += dwb;
            }
        }
    }
    pub fn line(&mut self, a: V3, b: V3, color: [f32; 3], opacity: f32) {
        let p = self.project(a);
        let q = self.project(b);
        if p.w < 0.35 || q.w < 0.35 || p.z > 1. || q.z > 1. {
            return;
        }
        let (dx, dy) = (q.x - p.x, q.y - p.y);
        let steps = dx.abs().max(dy.abs()).ceil().min(2500.) as usize;
        for i in 0..=steps {
            let t = i as f32 / steps.max(1) as f32;
            let (x, y) = ((p.x + dx * t).round() as i32, (p.y + dy * t).round() as i32);
            if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
                continue;
            }
            let at = y as usize * self.width + x as usize;
            let z = p.z + (q.z - p.z) * t;
            if z > self.depth[at] + 0.00001 {
                continue;
            }
            for (k, c) in color.iter().enumerate() {
                let i = at * 4 + k;
                self.pixels[i] = (self.pixels[i] as f32 * (1. - opacity) + c * opacity) as u8;
            }
        }
    }
    pub fn finish(&mut self) {
        if !self.options.ink {
            return;
        }
        let w = self.width;
        let h = self.height;
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let at = y * w + x;
                let id = self.ids[at];
                if id == 0 {
                    continue;
                }
                if self.ids[at + 1] != id || self.ids[at + w] != id {
                    let p = at * 4;
                    for k in 0..3 {
                        self.pixels[p + k] = (self.pixels[p + k] as f32
                            * (0.94 - 0.035 * self.options.watercolor))
                            as u8;
                    }
                    // A tiny wet edge bleeding into empty paper/sky; never covers another body.
                    if self.options.watercolor > 0. {
                        for neighbor in [at + 1, at + w] {
                            if self.ids[neighbor] == 0 {
                                for k in 0..3 {
                                    let n = neighbor * 4 + k;
                                    self.pixels[n] = (self.pixels[n] as f32 * 0.91
                                        + self.pixels[p + k] as f32 * 0.09)
                                        as u8;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{box_mesh, sphere};
    #[test]
    fn clips_near_and_side_planes() {
        let v = |x, y, z| ClipVertex([x, y, z, 1., 0.5, 0.5, 0., 0., 1., 0.]);
        assert!(clip_polygon(&[v(2., 0., 0.), v(3., 1., 0.), v(3., -1., 0.)]).is_empty());
        let p = clip_polygon(&[v(-0.5, -0.5, -2.), v(0.5, -0.5, 0.), v(0., 0.5, 0.)]);
        assert_eq!(p.len(), 4);
        for v in p {
            assert!(v.0[2] >= -v.0[3] - 0.0001);
        }
    }
    #[test]
    fn depth_order_and_sphere_coverage() {
        let mut r = Renderer::new(120, 100);
        let camera = Camera {
            eye: V3::new(0., 0., 6.),
            target: V3::ZERO,
            up: V3::Y,
            fov: 0.9,
        };
        let g = sphere(20, 12, None);
        let red = Material::glowing([230., 0., 0.]);
        let green = Material::glowing([0., 230., 0.]);
        r.begin(camera, 0.);
        r.draw(&g, M4::compose(V3::new(0., 0., -2.), 0., 1., 0.), red, 1);
        r.draw(&g, M4::identity(), green, 2);
        assert_eq!(r.ids[50 * 120 + 60], 2);
        assert!(r.pixels[(50 * 120 + 60) * 4 + 1] > 180);
        r.begin(camera, 0.);
        r.draw(&g, M4::identity(), green, 2);
        r.draw(&g, M4::compose(V3::new(0., 0., -2.), 0., 1., 0.), red, 1);
        for y in 42..59 {
            for x in 52..69 {
                assert_eq!(r.ids[y * 120 + x], 2, "hole at {x}, {y}");
            }
        }
    }
    #[test]
    fn culling_changes_work() {
        let mut r = Renderer::new(80, 60);
        r.begin(Camera::default(), 0.);
        let m = box_mesh(1., 1., 1.);
        let model = M4::compose(V3::new(9999., 0., 0.), 0., 1., 0.);
        r.draw(&m, model, Material::solid([255.; 3]), 2);
        assert_eq!(r.stats.culled, 1);
        assert_eq!(r.stats.triangles, 0);
        r.options.culling = false;
        r.draw(&m, model, Material::solid([255.; 3]), 2);
        assert_eq!(r.stats.triangles, 12);
    }
}
