mod camera;
mod geom;
mod obj;

use std::f32::consts::PI;

use anyhow::Result;
use camera::{perspective, Camera, UP};
use exr::prelude::*;
use geom::{Ray, Scene, SimpleScene};
use nalgebra::{Quaternion, UnitQuaternion, Vector3, Vector4};
use obj::save_obj;

use rayon::prelude::*;

pub type TVec3 = Vector3<f32>;

fn main() {
    if let Err(e) = real_main() {
        eprintln!("Error: {}", e);
    }
}

fn real_main() -> Result<()> {
    let scene = obj::load_obj("monkey.obj")?;
    // let scene = Scene {
    //     vertices: vec![
    //         Vector3::new(-1.0, 0.0, -1.0),
    //         Vector3::new(1.0, 0.0, -1.0),
    //         Vector3::new(0.0, 0.0, 1.0),
    //     ],
    //     triangles: vec![[0, 1, 2]],
    //     normals: vec![Vector3::new(0.0, 0.0, 1.0); 3],
    //     normal_triangles: vec![[0, 0, 0]],
    // };

    println!(
        "Loaded scene with {} vertices and {} triangles",
        scene.vertices.len(),
        scene.triangles.len()
    );

    let viewport_width = 1280;
    let viewport_height = 720;
    let aspect = viewport_width as f32 / viewport_height as f32;

    let camera_pos = Vector3::new(2.0, -3.0, 2.0);
    let camera_dir = UnitQuaternion::look_at_rh(&-camera_pos, &UP);
    let fov = 80.0 * PI / 180.0;

    let mut camera = Camera::new(
        camera_pos,
        camera_dir,
        perspective(fov, aspect),
    );

    // let mut debug_scene = scene.clone();

    let scene = scene.transform(&camera.extrinsic_matrix());
    let sun = Vector3::new(1.0, 0.0, 0.5).normalize();

    // let mut debug_scene = scene.clone();

    let scene: SimpleScene = scene.into();

    println!("Starting render");

    // let mut fb = vec![0.0; viewport_width * viewport_height * 3];

    let n_pixels = viewport_width * viewport_height;
    // for i in 0..n_pixels {
    let fb: Vec<_> = (0..n_pixels).into_par_iter().map(|i| {
    // let fb: Vec<_> = (0..n_pixels).map(|i| {
        let x = i % viewport_width;
        let y = i / viewport_width;
        let pixel_idx = (y * viewport_width + x) * 3;

        let x = x as f32;
        let y = y as f32;

        let viewport_width = viewport_width as f32;
        let viewport_height = viewport_height as f32;

        let ndc_x = (2.0 * x) / viewport_width - 1.0;
        let ndc_y = 1.0 - (2.0 * y) / viewport_height;
        let ndc_z = 1.0;

        let ndc_point = Vector4::new(ndc_x, ndc_y, ndc_z, 1.0);
        let inverse_proj = camera.projection.try_inverse().unwrap();

        let camera_space_point = inverse_proj * &ndc_point;
        let ray_dir = camera_space_point.normalize().xyz();

        let ray = Ray {
            origin: Vector3::new(0.0, 0.0, 0.0),
            direction: ray_dir,
        };

        // debug_scene.vertices.push(ray.direction);
        // let idx = debug_scene.vertices.len() as u32 - 1;
        // debug_scene.triangles.push([idx, idx, idx]);

        if let Some((_min_t, hit_idx)) = scene.intersects(&ray) {
            let normals = &scene.normals[hit_idx];
            let normal = (normals.a + normals.b + normals.c) / 3.0;

            let brightness = normal.dot(&sun).max(0.0);
            let b = brightness;
            // let b = 1.0;

            (b, b, b)
        } else {
            (0.0, 0.0, 0.0)
        }
    }).collect();

    // save_obj("debug.obj", &debug_scene)?;

    write_rgb_file(
        "output.exr",
        viewport_width as usize,
        viewport_height as usize,
        |x, y| {
            let idx = (y * viewport_width + x);
            fb[idx]
        },
    )?;

    Ok(())
}
