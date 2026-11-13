//! CPU geometry stage: OBJ triangulation and procedural primitive generation.
use crate::math::{M4, V3};
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub p: V3,
    pub n: V3,
    pub uv: [f32; 2],
}
#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<usize>,
    pub radius: f32,
}
impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<usize>) -> Self {
        let radius = vertices.iter().map(|v| v.p.length()).fold(0., f32::max);
        Self {
            vertices,
            indices,
            radius,
        }
    }
    pub fn recompute_normals(&mut self) {
        for v in &mut self.vertices {
            v.n = V3::ZERO;
        }
        for face in self.indices.chunks_exact(3) {
            let [a, b, c] = [face[0], face[1], face[2]];
            let n = (self.vertices[b].p - self.vertices[a].p)
                .cross(self.vertices[c].p - self.vertices[a].p)
                .normalized();
            for i in [a, b, c] {
                self.vertices[i].n += n;
            }
        }
        for v in &mut self.vertices {
            v.n = v.n.normalized();
        }
    }
    pub fn transform_vertices(&mut self, m: M4) {
        for v in &mut self.vertices {
            v.p = m.point(v.p);
            v.n = m.direction(v.n).normalized();
        }
        self.radius = self
            .vertices
            .iter()
            .map(|v| v.p.length())
            .fold(0., f32::max);
    }
}
pub fn sphere(segments: usize, rows: usize, deform: Option<&dyn Fn(V3) -> f32>) -> Mesh {
    let mut v = Vec::new();
    let mut idx = Vec::new();
    for j in 0..=rows {
        for i in 0..=segments {
            let (u, t) = (i as f32 / segments as f32, j as f32 / rows as f32);
            let (p, a) = (t * std::f32::consts::PI, u * std::f32::consts::TAU);
            let n = V3::new(p.sin() * a.cos(), p.cos(), p.sin() * a.sin());
            let r = deform.map_or(1., |f| f(n));
            v.push(Vertex {
                p: n * r,
                n,
                uv: [u, t],
            });
        }
    }
    for j in 0..rows {
        for i in 0..segments {
            let a = j * (segments + 1) + i;
            let b = a + segments + 1;
            idx.extend([a, a + 1, b, b, a + 1, b + 1]);
        }
    }
    let mut mesh = Mesh::new(v, idx);
    if deform.is_some() {
        mesh.recompute_normals();
    }
    mesh
}
pub fn box_mesh(w: f32, h: f32, d: f32) -> Mesh {
    let faces = [
        (
            V3::new(1., 0., 0.),
            [[1., -1., -1.], [1., 1., -1.], [1., 1., 1.], [1., -1., 1.]],
        ),
        (
            V3::new(-1., 0., 0.),
            [
                [-1., -1., 1.],
                [-1., 1., 1.],
                [-1., 1., -1.],
                [-1., -1., -1.],
            ],
        ),
        (
            V3::new(0., 1., 0.),
            [[-1., 1., -1.], [-1., 1., 1.], [1., 1., 1.], [1., 1., -1.]],
        ),
        (
            V3::new(0., -1., 0.),
            [
                [-1., -1., 1.],
                [-1., -1., -1.],
                [1., -1., -1.],
                [1., -1., 1.],
            ],
        ),
        (
            V3::new(0., 0., 1.),
            [[1., -1., 1.], [1., 1., 1.], [-1., 1., 1.], [-1., -1., 1.]],
        ),
        (
            V3::new(0., 0., -1.),
            [
                [-1., -1., -1.],
                [-1., 1., -1.],
                [1., 1., -1.],
                [1., -1., -1.],
            ],
        ),
    ];
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for (n, vs) in faces {
        let b = vertices.len();
        for (i, p) in vs.iter().enumerate() {
            vertices.push(Vertex {
                p: V3::new(p[0] * w * 0.5, p[1] * h * 0.5, p[2] * d * 0.5),
                n,
                uv: [
                    if i < 2 { 0. } else { 1. },
                    if i == 0 || i == 3 { 0. } else { 1. },
                ],
            });
        }
        indices.extend([b, b + 1, b + 2, b, b + 2, b + 3]);
    }
    Mesh::new(vertices, indices)
}
pub fn cone(radius: f32, height: f32, segments: usize, top: f32) -> Mesh {
    let mut v = Vec::new();
    let mut idx = Vec::new();
    for j in 0..2 {
        for i in 0..=segments {
            let u = i as f32 / segments as f32;
            let a = u * std::f32::consts::TAU;
            let r = if j == 0 { radius } else { top };
            v.push(Vertex {
                p: V3::new(a.cos() * r, j as f32 * height, a.sin() * r),
                n: V3::ZERO,
                uv: [u, j as f32],
            });
        }
    }
    for i in 0..segments {
        let a = i;
        let b = i + segments + 1;
        idx.extend([a, b, a + 1, b, b + 1, a + 1]);
    }
    let mut m = Mesh::new(v, idx);
    m.recompute_normals();
    m
}
pub fn ring(inner: f32, outer: f32, segments: usize) -> Mesh {
    let mut v = Vec::new();
    let mut idx = Vec::new();
    for j in 0..2 {
        for i in 0..=segments {
            let u = i as f32 / segments as f32;
            let a = u * std::f32::consts::TAU;
            let r = if j == 0 { inner } else { outer };
            v.push(Vertex {
                p: V3::new(a.cos() * r, 0., a.sin() * r),
                n: V3::Y,
                uv: [u, j as f32],
            });
        }
    }
    for i in 0..segments {
        let a = i;
        let b = i + segments + 1;
        idx.extend([a, a + 1, b, a + 1, b + 1, b]);
    }
    Mesh::new(v, idx)
}
pub fn parse_obj(text: &str) -> Result<Mesh, String> {
    let (mut positions, mut normals, mut uvs) =
        (Vec::<V3>::new(), Vec::<V3>::new(), Vec::<[f32; 2]>::new());
    let (mut vertices, mut indices) = (Vec::new(), Vec::new());
    let mut missing = false;
    let index = |s: &str, count: usize| -> Result<usize, String> {
        let n = s
            .parse::<isize>()
            .map_err(|_| format!("Invalid OBJ index: {s}"))?;
        let i = if n < 0 { count as isize + n } else { n - 1 };
        if i < 0 || i >= count as isize {
            return Err(format!("OBJ index out of bounds: {n}"));
        }
        Ok(i as usize)
    };
    for (line_no, line) in text.lines().enumerate() {
        let mut parts = line.split_whitespace();
        let Some(tag) = parts.next() else { continue };
        match tag {
            "v" | "vn" => {
                let p = parts
                    .take(3)
                    .map(|s| s.parse::<f32>())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| format!("Invalid vertex on line {}", line_no + 1))?;
                if p.len() != 3 {
                    return Err("Incomplete OBJ vertex".into());
                }
                let v = V3::new(p[0], p[1], p[2]);
                if tag == "v" {
                    positions.push(v)
                } else {
                    normals.push(v)
                }
            }
            "vt" => {
                let p = parts
                    .take(2)
                    .map(|s| s.parse::<f32>())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| "Invalid OBJ UV")?;
                if p.len() != 2 {
                    return Err("Incomplete OBJ UV".into());
                }
                uvs.push([p[0], p[1]]);
            }
            "f" => {
                let mut face = Vec::new();
                for p in parts {
                    let a = p.split('/').collect::<Vec<_>>();
                    let position = positions[index(a[0], positions.len())?];
                    let uv = if a.len() > 1 && !a[1].is_empty() {
                        uvs[index(a[1], uvs.len())?]
                    } else {
                        [0., 0.]
                    };
                    let n = if a.len() > 2 && !a[2].is_empty() {
                        normals[index(a[2], normals.len())?]
                    } else {
                        missing = true;
                        V3::ZERO
                    };
                    face.push(vertices.len());
                    vertices.push(Vertex { p: position, n, uv });
                }
                for i in 1..face.len().saturating_sub(1) {
                    indices.extend([face[0], face[i], face[i + 1]]);
                }
            }
            _ => {}
        }
    }
    if vertices.is_empty() {
        return Err("The OBJ contains no faces".into());
    }
    let mut mesh = Mesh::new(vertices, indices);
    if missing {
        mesh.recompute_normals();
    }
    Ok(mesh)
}
pub fn ship_mesh() -> Result<Mesh, String> {
    let mut m = parse_obj(include_str!("../assets/galaxy_one.obj"))?;
    let mut min = V3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let mut max = -min;
    for v in &m.vertices {
        min = V3::new(min.x.min(v.p.x), min.y.min(v.p.y), min.z.min(v.p.z));
        max = V3::new(max.x.max(v.p.x), max.y.max(v.p.y), max.z.max(v.p.z));
    }
    let center = (min + max) * 0.5;
    for v in &mut m.vertices {
        v.p = v.p - center;
    }
    m.transform_vertices(M4::compose(V3::ZERO, std::f32::consts::FRAC_PI_2, 1., 0.));
    Ok(m)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn outward_sphere() {
        let m = sphere(12, 8, None);
        for f in m.indices.chunks_exact(3) {
            let [a, b, c] = [m.vertices[f[0]].p, m.vertices[f[1]].p, m.vertices[f[2]].p];
            let n = (b - a).cross(c - a);
            if n.length() > 1e-5 {
                assert!(n.dot(a) > 0.);
            }
        }
    }
    #[test]
    fn obj_quad_and_negative_indices() {
        let m=parse_obj("v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nvt 0 0\nvt 1 0\nvt 1 1\nvt 0 1\nf -4/1 -3/2 -2/3 -1/4").unwrap();
        assert_eq!(m.indices.len(), 6);
        assert_eq!(m.vertices[2].uv, [1., 1.]);
    }
    #[test]
    fn model_supplied() {
        let m = ship_mesh().unwrap();
        assert!(m.indices.len() > 30);
        assert!(m.radius < 6.);
    }
}
