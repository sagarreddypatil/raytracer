use nalgebra::{DMatrix, Point3};

use crate::rng::{rand_circle, rand_f32};
use crate::scene::Scene;

use crate::{Point3f, Ray};

use rayon::prelude::*;

pub fn sample_once(scene: &Scene) -> DMatrix<f32> {
    let camera = &scene.camera;
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

        let jitter = rand_circle();
        let x = x + jitter.x;
        let y = y + jitter.y;

        let ndc_x = (2.0 * x) / viewport_width - 1.0;
        let ndc_y = 1.0 - (2.0 * y) / viewport_height;
        let ndc_z = 1.0;

        let ndc_point = Point3f::new(ndc_x, ndc_y, ndc_z);

        let camera_space_point = camera.inv_projection.transform_point(&ndc_point);
        let ray_dir = camera_space_point.coords;

        let wow = camera.transform.inv_matrix_f;
        let ray_dir = wow.transform_vector(&ray_dir);
        let ray_dir = wow.transform_vector(&ray_dir);

        let ray = Ray::new(Point3::new(0.0, 0.0, 0.0), ray_dir);

        let b = scene.sample(&ray, 4);
        b
    }).collect();

    DMatrix::from_vec(camera.width, camera.height, fb)
}
