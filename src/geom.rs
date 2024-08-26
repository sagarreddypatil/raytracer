use crate::camera::Camera;
use cgmath::{InnerSpace, Matrix4, Vector3};
use rayon::prelude::*;

pub struct Ray {
    pub origin: Vector3<f32>,
    pub direction: Vector3<f32>,
}

pub struct Triangle {
    pub a: Vector3<f32>,
    pub b: Vector3<f32>,
    pub c: Vector3<f32>,
}

// from wikipedia
fn moller_trumbore_intersection(
    origin: &Vector3<f32>,
    direction: &Vector3<f32>,
    triangle: &Triangle,
) -> Option<f32> {
    let a = &triangle.a;
    let b = &triangle.b;
    let c = &triangle.c;

    let e1 = b - a;
    let e2 = c - a;

    let ray_cross_e2 = direction.cross(e2);
    let det = e1.dot(ray_cross_e2);

    if det > -f32::EPSILON && det < f32::EPSILON {
        return None; // This ray is parallel to this triangle.
    }

    let inv_det = 1.0 / det;
    let s = origin - a;
    let u = inv_det * s.dot(ray_cross_e2);
    if u < 0.0 || u > 1.0 {
        return None;
    }

    let s_cross_e1 = s.cross(e1);
    let v = inv_det * direction.dot(s_cross_e1);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    // At this stage we can compute t to find out where the intersection point is on the line.
    let t = inv_det * e2.dot(s_cross_e1);

    if t > f32::EPSILON {
        // ray intersection
        return Some(t);
    } else {
        // This means that there is a line intersection but not a ray intersection.
        return None;
    }
}

impl Ray {
    pub fn intersects<'a>(&self, triangle: &Triangle) -> Option<f32> {
        moller_trumbore_intersection(&self.origin, &self.direction, triangle)
    }
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub vertices: Vec<Vector3<f32>>,
    pub triangles: Vec<[u32; 3]>,

    pub normals: Vec<Vector3<f32>>,
    pub normal_triangles: Vec<[u32; 3]>,

    // materials: Vec<Material>,
}

impl Scene {
    pub fn transform(&self, matrix: &Matrix4<f32>) -> Scene {
        let mut vertices = Vec::new();
        for vertex in &self.vertices {
            vertices.push((matrix * vertex.extend(1.0)).truncate());
        }

        Scene {
            vertices,
            triangles: self.triangles.clone(),

            normals: self.normals.clone(),
            normal_triangles: self.normal_triangles.clone(),
        }
    }
}

pub struct SimpleScene {
    pub triangles: Vec<Triangle>,
    pub normals: Vec<Triangle>,
}

impl From<Scene> for SimpleScene {
    fn from(scene: Scene) -> Self {
        let mut triangles = Vec::new();
        let mut normals = Vec::new();

        for triangle in &scene.triangles {
            let a = scene.vertices[triangle[0] as usize];
            let b = scene.vertices[triangle[1] as usize];
            let c = scene.vertices[triangle[2] as usize];
            triangles.push(Triangle { a, b, c });
        }

        for normal_triangle in &scene.normal_triangles {
            let a = scene.normals[normal_triangle[0] as usize];
            let b = scene.normals[normal_triangle[1] as usize];
            let c = scene.normals[normal_triangle[2] as usize];
            normals.push(Triangle { a, b, c });
        }

        SimpleScene { triangles, normals }
    }
}

impl SimpleScene {
    pub fn intersects(&self, ray: &Ray) -> Option<(f32, usize)> {
        let mut min_t = f32::INFINITY;
        let mut hit_idx = None;

        for (i, triangle) in self.triangles.iter().enumerate() {
            if let Some(t) = ray.intersects(triangle) {
                if t < min_t {
                    min_t = t;
                    hit_idx = Some(i);
                }
            }
        }

        hit_idx.map(|idx| (min_t, idx))
    }

    // rayon version
    // pub fn intersects(&self, ray: &Ray) -> Option<(f32, usize)> {
    //     let it = self.triangles.par_iter().enumerate().filter_map(|(i, triangle)| {
    //         ray.intersects(triangle).map(|t| (t, i))
    //     });

    //     it.min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
    // }
}
