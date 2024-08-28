use nalgebra::{DMatrix, Point3, Vector4};

use crate::{
    camera::Camera,
    geom::{BVHScene, TRay},
    texture::{equirectangular, Texture},
    TVec3,
};

use rayon::prelude::*;

fn simulate_ray(ray: TRay, camera: &Camera, scene: &BVHScene, hdri: &DMatrix<f32>) -> f32 {
    if let Some((dist, tri_idx)) = scene.intersects(&ray) {
        let tri = &scene.triangles[tri_idx];
        let sun = camera.vector_world_to_camera(TVec3::new(1.0, 0.0, 1.0).normalize());

        let tri_normals = &scene.normals[tri_idx];
        let normal = (tri_normals.a + tri_normals.b + tri_normals.c) / 3.0;

        let brightness = normal.dot(&sun);
        if brightness > 0.0 {
            brightness
        } else {
            0.0
        }
    } else {
        let extrinsic_inv = camera.inverse_extrinsic_matrix();
        let extrinsic_inv_rot = extrinsic_inv.fixed_view::<3, 3>(0, 0);

        let hdri_world = extrinsic_inv_rot * ray.direction;

        let hdri_uv = equirectangular(hdri_world);
        let val = hdri.sample_nearest(hdri_uv);

        val
    }
}

pub fn sample(scene: &BVHScene, camera: &Camera, hdri: &DMatrix<f32>) -> DMatrix<f32> {
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

        let x = x + (1.0 * rand::random::<f32>() - 0.5);
        let y = y + (1.0 * rand::random::<f32>() - 0.5);

        let ndc_x = (2.0 * x) / viewport_width - 1.0;
        let ndc_y = 1.0 - (2.0 * y) / viewport_height;
        let ndc_z = 1.0;

        let ndc_point = Vector4::new(ndc_x, ndc_y, ndc_z, 1.0);
        let inverse_proj = camera.inverse_projection_matrix();

        let camera_space_point = inverse_proj * &ndc_point;
        let ray_dir = camera_space_point.xyz();
        let ray = TRay::new(Point3::new(0.0, 0.0, 0.0), ray_dir);

        let b = simulate_ray(ray, camera, scene, hdri);
        b
    }).collect();

    DMatrix::from_vec(camera.width, camera.height, fb)
}
