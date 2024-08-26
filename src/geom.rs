use crate::TVec3;
use nalgebra::{Matrix4, Point3};
use bvh::ray::Ray;

pub type TRay = Ray<f32, 3>;

pub struct Triangle {
    pub a: TVec3,
    pub b: TVec3,
    pub c: TVec3,
}

impl Triangle {
    pub fn new(a: TVec3, b: TVec3, c: TVec3) -> Self {
        Triangle { a, b, c }
    }
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub vertices: Vec<TVec3>,
    pub triangles: Vec<[u32; 3]>,

    pub normals: Vec<TVec3>,
    pub normal_triangles: Vec<[u32; 3]>,
    // materials: Vec<Material>,
}

impl Scene {
    pub fn transform(&self, matrix: &Matrix4<f32>) -> Scene {
        let mut vertices = Vec::new();
        for vertex in &self.vertices {
            let vertex = nalgebra::Vector4::new(vertex.x, vertex.y, vertex.z, 1.0);
            vertices.push((matrix * vertex).xyz());
        }

        Scene {
            vertices,
            triangles: self.triangles.clone(),

            normals: self.normals.clone(),
            normal_triangles: self.normal_triangles.clone(),
        }
    }
}

use bvh::aabb::{Aabb, Bounded};
use bvh::bounding_hierarchy::BHShape;
use bvh::bvh::Bvh;

#[derive(Debug, Clone)]
pub struct BVHTriangle {
    a: Point3<f32>,
    b: Point3<f32>,
    c: Point3<f32>,

    aabb: Aabb<f32, 3>,
    node_index: usize,

    pub arr_index: usize,
}

impl PartialEq for BVHTriangle {
    fn eq(&self, other: &Self) -> bool {
        self.a == other.a && self.b == other.b && self.c == other.c
    }
}

impl BVHTriangle {
    pub fn new(a: TVec3, b: TVec3, c: TVec3) -> Self {
        let a: Point3<f32> = a.into();
        let b: Point3<f32> = b.into();
        let c: Point3<f32> = c.into();

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

pub struct BVHScene {
    bvh: Bvh<f32, 3>,
    pub triangles: Vec<BVHTriangle>,
    pub normals: Vec<Triangle>,
}

impl From<Scene> for BVHScene {
    fn from(scene: Scene) -> Self {
        let mut triangles = Vec::new();
        let mut normals = Vec::new();

        for triangle in &scene.triangles {
            let a = scene.vertices[triangle[0] as usize];
            let b = scene.vertices[triangle[1] as usize];
            let c = scene.vertices[triangle[2] as usize];
            let mut tri = BVHTriangle::new(a, b, c);
            tri.arr_index = triangles.len();
            triangles.push(tri);
        }

        for normal_triangle in &scene.normal_triangles {
            let a = scene.normals[normal_triangle[0] as usize];
            let b = scene.normals[normal_triangle[1] as usize];
            let c = scene.normals[normal_triangle[2] as usize];
            normals.push(Triangle::new(a, b, c));
        }

        let bvh = Bvh::build(&mut triangles);
        BVHScene {
            bvh,
            triangles,
            normals,
        }
    }
}

impl BVHScene {
    pub fn intersects(&self, ray: &TRay) -> Option<(f32, usize)> {
        // let ray: Ray = ray.clone();
        // let ray = ray.into();
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
