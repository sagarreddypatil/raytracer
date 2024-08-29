use std::hash::RandomState;

use nalgebra::{DMatrix, Vector4};

use crate::camera::Camera;
use crate::geom::{normalize, BVHTriangle, BvhScene, Object};
use crate::rng::{rand_direction, rand_f32};
use crate::texture::{equirectangular, Texture};
use crate::{Matrix4f, Point3d, Ray, Vector2f, Vector3d, Vector3f};

pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<Object>,
    pub env_map: DMatrix<f32>,

    pub bvh: Option<BvhScene>,
}

impl Scene {
    pub fn new(camera: Camera, objects: Vec<Object>, env_map: DMatrix<f32>) -> Self {
        Self {
            camera,
            objects,
            env_map,
            bvh: None,
        }
    }

    fn sample_env(&self, ray: &Ray) -> f32 {
        let ray_world = self
            .camera
            .transform
            .matrix_f
            .transform_vector(&ray.direction);

        // let hdri_uv = equirectangular(ray_world);
        // let val = self.env_map.sample_nearest(hdri_uv);

        ray_world.y
    }

    pub fn sample(&self, ray: &Ray, max_bounces: u32) -> f32 {
        assert!(self.bvh.is_some());
        let bvh = self.bvh.as_ref().unwrap();

        if max_bounces == 0 {
            return 0.0;
        }

        if let Some((dist, tri_idx)) = bvh.intersects(ray) {
            let new_origin = ray.origin + ray.direction * dist;

            let tri = &bvh.triangles[tri_idx];
            let (alpha, beta) = tri.barycentric(new_origin);

            let tri_normals = &bvh.normals[tri_idx];
            let normal = tri_normals.0 * (1.0 - alpha - beta)
                + tri_normals.1 * alpha
                + tri_normals.2 * beta;

            // specular reflection
            // let new_dir = ray.direction - 2.0 * ray.direction.dot(&normal) * normal;
            // let new_ray = Ray::new(new_origin, new_dir);

            // diffuse reflection
            let dir = normalize(normal + rand_direction());
            // let ray = Ray::new(new_origin, dir);
            let ray = Ray {
                origin: new_origin,
                direction: dir,
                inv_direction: Vector3f::new(1.0 / dir.x, 1.0 / dir.y, 1.0 / dir.z),
            };

            self.sample(&ray, max_bounces - 1)
        } else {
            self.sample_env(ray)
        }
    }

    pub fn build_bvh(&mut self) {
        self.bvh = Some(self.bvh());
    }

    fn bvh(&self) -> BvhScene {
        let world_to_camera = self.camera.transform.inv_matrix;

        let mut triangles = Vec::new();
        let mut normals = Vec::new();

        for object in &self.objects {
            let object_to_world = object.transform.matrix;
            let object_to_camera = world_to_camera * object_to_world;

            // This casting business is done because
            // we want to allow for potentially large objects (such as your mom)
            // so we cast the vertices to f64 before transforming
            // and then cast them back to f32

            let transform_vertex = |i: u32| {
                let vertex = &object.mesh.vertices[i as usize];
                let vertex = vertex.cast();

                let vertex = object_to_camera.transform_point(&vertex);

                vertex.cast()
            };

            for triangle in &object.mesh.triangles {
                let a = transform_vertex(triangle[0]);
                let b = transform_vertex(triangle[1]);
                let c = transform_vertex(triangle[2]);

                let mut tri = BVHTriangle::new(a, b, c);
                tri.arr_index = triangles.len();
                triangles.push(tri);
            }

            // funny casting business not needed for normals
            // so we just cast the matrix to f32
            let object_to_camera: Matrix4f = object_to_camera.matrix().cast();

            for normal_triangle in &object.mesh.normal_triangles {
                let a = &object.mesh.normals[normal_triangle[0] as usize];
                let b = &object.mesh.normals[normal_triangle[1] as usize];
                let c = &object.mesh.normals[normal_triangle[2] as usize];

                let a = object_to_camera.transform_vector(&a);
                let b = object_to_camera.transform_vector(&b);
                let c = object_to_camera.transform_vector(&c);

                normals.push((a, b, c));
            }
        }

        BvhScene::new(triangles, normals)
    }
}
