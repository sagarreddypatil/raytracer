use nalgebra::{Point3, Vector4};

use crate::{
    camera::Camera,
    geom::{BVHScene, TRay},
    TVec3,
};

use rayon::prelude::*;

fn simulate_ray(ray: TRay, scene: &BVHScene, sun: TVec3) -> f32 {
    if let Some((t, hit_idx)) = scene.intersects(&ray) {
        let normals = &scene.normals[hit_idx];
        let normal = (normals.a + normals.b + normals.c) / 3.0;

        // another ray to the sun
        let intersect_pos = ray.origin + ray.direction * t;
        let sun_ray = TRay::new(intersect_pos - (sun * 1e-4), sun);

        if let Some((_, _)) = scene.intersects(&sun_ray) {
            0.0
        } else {
            let brightness = normal.dot(&sun).max(0.0);
            let b = brightness;

            b
        }
    } else {
        0.0
    }
}

pub fn sample(scene: &BVHScene, camera: &Camera, sun: TVec3) -> Vec<(f32, f32, f32, f32)> {
    let n_pixels = camera.width * camera.height;

    let viewport_width = camera.width as f32;
    let viewport_height = camera.height as f32;

    #[rustfmt::skip]
    let fb: Vec<_> = (0..n_pixels).into_par_iter().map(|i| {
    // let fb: Vec<_> = (0..n_pixels).map(|i| {
        let x = i % viewport_width as usize;
        let y = i / viewport_width as usize;

        let x = x as f32;
        let y = y as f32;

        let ndc_x = (2.0 * x) / viewport_width - 1.0;
        let ndc_y = 1.0 - (2.0 * y) / viewport_height;
        let ndc_z = 1.0;

        let ndc_point = Vector4::new(ndc_x, ndc_y, ndc_z, 1.0);
        let inverse_proj = camera.inverse_projection_matrix();

        let camera_space_point = inverse_proj * &ndc_point;
        let ray_dir = camera_space_point.xyz();
        let ray = TRay::new(Point3::new(0.0, 0.0, 0.0), ray_dir);

        let b = simulate_ray(ray, scene, sun);
        (b, b, b, 1.0)
    }).collect();

    fb
}
