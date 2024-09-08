use std::f32::consts::PI;
use std::f32::EPSILON;

use nalgebra::DMatrix;

use crate::bsdf::{Glossy, Lambertian, BSDF, UP};
use crate::camera::Camera;
use crate::geom::{normalize, BVHTriangle, BvhScene, Object};
use crate::rng::{rand_direction, rand_hemisphere};
use crate::texture::{equirectangular, Texture};
use crate::{bsdf, rad, Color, Matrix3f, Matrix4f, Ray, Vector3f};

pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<Object>,
    pub env_map: DMatrix<Color>,

    pub bvh: Option<BvhScene>,
}

fn close_to_zero(val: f32) -> bool {
    val.abs() < EPSILON
}

impl Scene {
    pub fn new(camera: Camera, objects: Vec<Object>, env_map: DMatrix<Color>) -> Self {
        Self {
            camera,
            objects,
            env_map,
            bvh: None,
        }
    }

    fn sample_env(&self, ray: &Ray) -> Color {
        let ray_world = self
            .camera
            .transform
            .matrix_f
            .transform_vector(&ray.direction);

        let hdri_uv = equirectangular(ray_world);
        let val = self.env_map.sample_linear(hdri_uv);

        val
        // 0.5

        // ray_world.y.max(0.0)

        // make a sun disc
        // let sun_dir = Vector3f::new(0.0, 1.0, 0.5).normalize();
        // let sun_intensity = 5.0;

        // let cos = sun_dir.dot(&ray_world);
        // let angle = cos.acos();

        // let sun_size_deg = 45.0;
        // let sun_size = sun_size_deg * PI / 180.0;

        // if angle < sun_size {
        //     sun_intensity
        // } else {
        //     0.0
        // }
    }

    pub fn sample(&self, ray: &Ray, max_bounces: u32) -> Color {
        assert!(self.bvh.is_some());
        let bvh = self.bvh.as_ref().unwrap();

        if max_bounces == 0 {
            return Color::zeros();
        }

        if let Some((dist, tri_idx)) = bvh.intersects(ray) {
            let new_origin = ray.origin + ray.direction * dist;

            let tri = &bvh.triangles[tri_idx];
            let (alpha, beta) = tri.barycentric(new_origin);

            let tri_normals = &bvh.normals[tri_idx];
            let normal =
                tri_normals.0 * (1.0 - alpha - beta) + tri_normals.1 * alpha + tri_normals.2 * beta;

            // assert!(normal.norm() - 1.0 < 1e-4);

            let basis_z = normal.normalize();
            let basis_y = Vector3f::new(1.0, 0.0, 0.0).cross(&basis_z).normalize();
            let basis_x = basis_y.cross(&basis_z).normalize();

            let from_normal = Matrix3f::from_columns(&[basis_x, basis_y, basis_z]);
            let to_normal = from_normal.transpose();

            // ray.direction.z

            // specular reflection
            // let dir = ray.direction - 2.0 * ray.direction.dot(&normal) * normal;
            // let ray = Ray::new(new_origin, dir);

            if ray.direction.dot(&normal) > 0.0 {
                // backface culling
                return Color::zeros();
            }

            // diffuse reflection
            // let dir = normalize(normal + rand_direction());
            // let ray = Ray {
            //     origin: new_origin + dir * 1e-4,
            //     direction: dir,
            //     inv_direction: Vector3f::new(1.0 / dir.x, 1.0 / dir.y, 1.0 / dir.z),
            // };

            // self.sample(&ray, max_bounces - 1)

            // bsdf based rendering
            // let bsdf = Lambertian { albedo: 0.5 };
            let bsdf = Lambertian { albedo: 0.9 };

            // enter normal space
            let reflected = to_normal * ray.direction;
            let incedent = bsdf.sample(reflected);

            let dot_component = incedent.dot(&UP); // dot product w/ normal in rendering equation
            let pdf = bsdf.pdf(incedent, reflected);
            let value = bsdf.value(incedent, reflected);

            if dot_component <= EPSILON {
                // this ray contributes nothing
                return Color::zeros();
            }

            let dir = normalize(from_normal * incedent);
            // leave normal space

            let ray = Ray {
                origin: new_origin + dir * 1e-4,
                direction: dir,
                inv_direction: Vector3f::new(1.0 / dir.x, 1.0 / dir.y, 1.0 / dir.z),
            };

            // L component of rendering equation
            let output = self.sample(&ray, max_bounces - 1);

            // final rendering equation f * L * (dot) / pdf
            let coeff = if close_to_zero(value) && close_to_zero(pdf) {
                // this means that pdf is zero everywhere except at one point (like in the case of
                // glossy bsdfs) so we can just return the value at that point

                1.0
            } else {
                dot_component * value / pdf
            };

            output * coeff
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
