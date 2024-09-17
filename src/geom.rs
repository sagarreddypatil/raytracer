use std::fmt::Debug;

use nalgebra::Point3;

use bvh::aabb::{Aabb, Bounded};
use bvh::bounding_hierarchy::BHShape;
use bvh::bvh::Bvh;

use crate::{Affine, Matrix4d, Matrix4f, Point3d, Point3f, Quaternion, Ray, Vector3d, Vector3f};

pub struct Object {
    pub transform: Transform,
    pub mesh: Mesh,
    pub material: Material,
}

impl Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Object")
            .field("transform", &self.transform)
            .finish()
    }
}

pub struct Mesh {
    pub vertices: Vec<Point3f>,
    pub normals: Vec<Vector3f>,

    pub triangles: Vec<[u32; 3]>,
    pub normal_triangles: Vec<[u32; 3]>,
}

pub struct Transform {
    pub position: Point3d,
    pub rotation: Quaternion,
    pub scale: Vector3d,

    pub matrix: Affine,
    pub inv_matrix: Affine,

    pub matrix_f: Matrix4f,
    pub inv_matrix_f: Matrix4f,
}

impl Debug for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transform")
            .field("position", &self.position)
            .field("rotation", &self.rotation)
            .field("scale", &self.scale)
            .finish()
    }
}

impl Transform {
    pub fn identity() -> Self {
        Self::new(
            Point3d::origin(),
            Quaternion::identity(),
            Vector3d::new(1.0, 1.0, 1.0),
        )
    }

    pub fn new(position: Point3d, rotation: Quaternion, scale: Vector3d) -> Self {
        let m_scale = Affine::from_matrix_unchecked(Matrix4d::new_nonuniform_scaling(&scale));
        let m_rotation = Affine::from_matrix_unchecked(Matrix4d::from(rotation));
        let m_translation =
            Affine::from_matrix_unchecked(Matrix4d::new_translation(&position.coords));

        let matrix = m_translation * m_scale * m_rotation;
        let inv_matrix = matrix.inverse();

        let matrix_f = matrix.matrix().cast();
        let inv_matrix_f = inv_matrix.matrix().cast();

        Self {
            position,
            rotation,
            scale,

            matrix,
            inv_matrix,
            matrix_f,
            inv_matrix_f,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BVHTriangle {
    pub a: Point3f,
    pub b: Point3f,
    pub c: Point3f,

    aabb: Aabb<f32, 3>,
    node_index: usize,

    pub arr_index: usize,
}

impl BVHTriangle {
    pub fn new(a: Point3f, b: Point3f, c: Point3f) -> Self {
        let min_bound = Point3::new(
            a.x.min(b.x).min(c.x),
            a.y.min(b.y).min(c.y),
            a.z.min(b.z).min(c.z),
        );

        let max_bound = Point3::new(
            a.x.max(b.x).max(c.x),
            a.y.max(b.y).max(c.y),
            a.z.max(b.z).max(c.z),
        );

        let aabb = Aabb::with_bounds(min_bound, max_bound);

        Self {
            a,
            b,
            c,
            aabb,
            node_index: 0,
            arr_index: 0,
        }
    }

    pub fn barycentric(&self, point: Point3f) -> (f32, f32) {
        let v0 = self.b - self.a;
        let v1 = self.c - self.a;
        let v2 = point - self.a;

        let d00 = v0.dot(&v0);
        let d01 = v0.dot(&v1);
        let d11 = v1.dot(&v1);
        let d20 = v2.dot(&v0);
        let d21 = v2.dot(&v1);

        let denom = d00 * d11 - d01 * d01;
        let beta = (d11 * d20 - d01 * d21) / denom;
        let gamma = (d00 * d21 - d01 * d20) / denom;

        (beta, gamma)
    }
}

impl Bounded<f32, 3> for BVHTriangle {
    fn aabb(&self) -> Aabb<f32, 3> {
        self.aabb
    }
}

impl BHShape<f32, 3> for BVHTriangle {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Material {
    Diffuse(f32),
    Glossy,
}

pub struct BvhScene {
    bvh: Bvh<f32, 3>,
    pub triangles: Vec<BVHTriangle>,
    pub normals: Vec<(Vector3f, Vector3f, Vector3f)>,
    pub materials: Vec<Material>,
}

// copy pasted from https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm
fn ray_triangle_intersection(ray: &Ray, a: Point3f, b: Point3f, c: Point3f) -> Option<f32> {
    let origin = ray.origin;
    let direction = ray.direction;

    let e1 = b - a;
    let e2 = c - a;

    let ray_cross_e2 = direction.cross(&e2);
    let det = e1.dot(&ray_cross_e2);

    if det > -f32::EPSILON && det < f32::EPSILON {
        return None; // This ray is parallel to this triangle.
    }

    let inv_det = 1.0 / det;
    let s = origin - a;
    let u = inv_det * s.dot(&ray_cross_e2);
    if u < 0.0 || u > 1.0 {
        return None;
    }

    let s_cross_e1 = s.cross(&e1);
    let v = inv_det * direction.dot(&s_cross_e1);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    // At this stage we can compute t to find out where the intersection point is on the line.
    let t = inv_det * e2.dot(&s_cross_e1);

    if t > f32::EPSILON {
        // ray intersection
        // let intersection_point = origin + direction * t;
        return Some(t);
    } else {
        // This means that there is a line intersection but not a ray intersection.
        return None;
    }
}

impl BvhScene {
    pub fn new(
        mut triangles: Vec<BVHTriangle>,
        normals: Vec<(Vector3f, Vector3f, Vector3f)>,
        materials: Vec<Material>,
    ) -> Self {
        let bvh = Bvh::build(&mut triangles);

        Self {
            bvh,
            triangles,
            normals,
            materials,
        }
    }

    pub fn intersects(&self, ray: &Ray) -> Option<(f32, usize)> {
        let hits = self.bvh.traverse(&ray, &self.triangles);
        let mut min_t = f32::INFINITY;
        let mut hit_idx = None;

        for triangle in hits.into_iter() {
            // let intersection = ray.intersects_triangle(&triangle.a, &triangle.b, &triangle.c);
            // if intersection.distance.is_finite() && intersection.distance < min_t {
            //     min_t = intersection.distance;

            let distance = ray_triangle_intersection(ray, triangle.a, triangle.b, triangle.c);
            if let Some(t) = distance {
                if t < min_t {
                    min_t = t;

                    let i = triangle.arr_index;
                    hit_idx = Some(i);
                }
            }
        }

        hit_idx.map(|idx| (min_t, idx))
    }
}

fn inv_sqrt(x: f32) -> f32 {
    let i = x.to_bits();
    let i = 0x5f3759df - (i >> 1);
    let y = f32::from_bits(i);

    y * (1.5 - 0.5 * x * y * y)
}

pub fn normalize(v: Vector3f) -> Vector3f {
    let len = inv_sqrt(v.norm_squared());
    v * len
}
