use nalgebra::Point3;

use bvh::aabb::{Aabb, Bounded};
use bvh::bounding_hierarchy::BHShape;
use bvh::bvh::Bvh;

use crate::{Affine, Matrix4d, Matrix4f, Point3d, Point3f, Quaternion, Ray, Vector3d, Vector3f};

pub struct Object {
    pub transform: Transform,
    pub mesh: Mesh,
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

        let matrix = m_translation * m_rotation * m_scale;
        let inv_matrix = matrix.inverse();
        let matrix_f = matrix.to_homogeneous().cast();
        let inv_matrix_f = inv_matrix.to_homogeneous().cast();

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
    a: Point3f,
    b: Point3f,
    c: Point3f,

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

pub struct BvhScene {
    bvh: Bvh<f32, 3>,
    pub triangles: Vec<BVHTriangle>,
    pub normals: Vec<(Vector3f, Vector3f, Vector3f)>,
}

impl BvhScene {
    pub fn new(
        mut triangles: Vec<BVHTriangle>,
        normals: Vec<(Vector3f, Vector3f, Vector3f)>,
    ) -> Self {
        let bvh = Bvh::build(&mut triangles);

        Self {
            bvh,
            triangles,
            normals,
        }
    }

    pub fn intersects(&self, ray: &Ray) -> Option<(f32, usize)> {
        let hits = self.bvh.traverse(&ray, &self.triangles);
        let mut min_t = f32::INFINITY;
        let mut hit_idx = None;

        for triangle in hits.into_iter() {
            let intersection = ray.intersects_triangle(&triangle.a, &triangle.b, &triangle.c);
            if intersection.distance.is_finite() && intersection.distance < min_t {
                min_t = intersection.distance;

                let i = triangle.arr_index;
                hit_idx = Some(i);
            }
        }

        hit_idx.map(|idx| (min_t, idx))
    }
}
