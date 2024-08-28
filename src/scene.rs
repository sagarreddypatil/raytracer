use nalgebra::DMatrix;

use crate::camera::Camera;
use crate::geom::{BVHTriangle, BvhScene, Object};
use crate::texture::{equirectangular, Texture};
use crate::{Matrix4f, Ray, Vector3f};

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
            .inv_matrix_f
            .transform_vector(&ray.direction);

        let hdri_uv = equirectangular(ray_world);
        let val = self.env_map.sample_nearest(hdri_uv);

        val
    }

    pub fn sample(&self, ray: &Ray, max_bounces: u32) -> f32 {
        assert!(self.bvh.is_some());
        let bvh = self.bvh.as_ref().unwrap();

        if max_bounces == 0 {
            return 0.0;
        }

        if let Some((dist, tri_idx)) = bvh.intersects(ray) {
            let sun = self
                .camera
                .transform
                .inv_matrix_f
                .transform_vector(&Vector3f::new(1.0, 0.0, 1.0))
                .normalize();

            let tri_normals = &bvh.normals[tri_idx];
            let normal = (tri_normals.0 + tri_normals.1 + tri_normals.2) / 3.0;

            let brightness = normal.dot(&sun);
            if brightness > 0.0 {
                brightness
            } else {
                0.0
            }
        } else {
            self.sample_env(ray)
        }
    }

    pub fn build_bvh(&mut self) {
        self.bvh = Some(self.bvh());
    }

    fn bvh(&self) -> BvhScene {
        let world_to_camera = self.camera.transform.matrix.inverse();

        let mut triangles = Vec::new();
        let mut normals = Vec::new();

        for object in &self.objects {
            let object_to_world = object.transform.matrix;
            let object_to_camera = world_to_camera * object_to_world;

            for triangle in &object.mesh.triangles {
                // This casting business is done because
                // we want to allow for potentially large objects
                // so we cast the vertices to f64 before transforming
                // and then cast them back to f32

                let a = &object.mesh.vertices[triangle[0] as usize];
                let b = &object.mesh.vertices[triangle[1] as usize];
                let c = &object.mesh.vertices[triangle[2] as usize];

                let a = a.cast();
                let b = b.cast();
                let c = c.cast();

                let a = object_to_camera.transform_point(&a);
                let b = object_to_camera.transform_point(&b);
                let c = object_to_camera.transform_point(&c);

                let a = a.cast();
                let b = b.cast();
                let c = c.cast();

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
