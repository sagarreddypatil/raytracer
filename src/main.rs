mod camera;
mod geom;
mod obj;

use std::{arch::x86_64, f32::consts::PI};

use anyhow::Result;
use camera::{perspective, Camera, UP};
use cgmath::{InnerSpace, Quaternion, Rad, Rotation, Rotation3, SquareMatrix, Vector3};
use exr::prelude::*;
use geom::{Ray, Scene, Triangle};
use obj::save_obj;

fn main() {
    if let Err(e) = real_main() {
        eprintln!("Error: {}", e);
    }
}

fn real_main() -> Result<()> {
    let scene = obj::load_obj("monkey.obj")?;

    // let scene = Scene {
    //     vertices: vec![
    //         Vector3::new(-1.0, 0.0, 1.0),
    //         Vector3::new(1.0, 0.0, 1.0),
    //         Vector3::new(-1.0, 0.0, -1.0),
    //         Vector3::new(1.0, 0.0, -1.0),
    //     ],
    //     triangles: vec![[0, 2, 1], [1, 2, 3]],
    // };

    // let scene = Scene {
    //     vertices: vec![
    //         Vector3::new(-1.0, 0.0, -1.0),
    //         Vector3::new(1.0, 0.0, -1.0),
    //         Vector3::new(0.0, 0.0, 1.0),
    //     ],
    //     triangles: vec![[0, 1, 2]],
    // };

    println!(
        "Loaded scene with {} vertices and {} triangles",
        scene.vertices.len(),
        scene.triangles.len()
    );

    let camera_pos = Vector3::new(0.0, 5.0, 0.0);
    let fov = 60.0 * PI / 180.0;
    let mut camera = Camera::new(
        camera_pos,
        Quaternion::look_at(-camera_pos, UP),
        // Quaternion::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), Rad(PI)),
        perspective(fov, 16.0 / 9.0),
    );

    // let mut debug_scene = scene.clone();

    let scene = scene.transform(&camera.extrinsic_matrix());
    let sun = Vector3::new(-1.0, 0.0, 0.0).normalize();

    // let mut debug_scene = scene.clone();

    let viewport_width = 1280;
    let viewport_height = 720;

    let mut fb = vec![0.0; viewport_width * viewport_height * 3];

    for y in 0..viewport_height {
        for x in 0..viewport_width {
            let pixel_idx = (y * viewport_width + x) * 3;

            let x = x as f32;
            let y = y as f32;

            let viewport_width = viewport_width as f32;
            let viewport_height = viewport_height as f32;

            let ndc_x = (2.0 * x) / viewport_width - 1.0;
            let ndc_y = 1.0 - (2.0 * y) / viewport_height;
            let ndc_z = 1.0;

            let ndc_point = Vector3::new(ndc_x, ndc_y, ndc_z);
            let inverse_proj = camera.projection.invert().unwrap();

            let camera_space_point = inverse_proj * ndc_point.extend(1.0);
            let ray_dir = camera_space_point.truncate().normalize();

            let ray = Ray {
                origin: Vector3::new(0.0, 0.0, 0.0),
                direction: ray_dir,
            };

            // debug_scene.vertices.push(ray.direction);
            // let idx = debug_scene.vertices.len() as u32 - 1;
            // debug_scene.triangles.push([idx, idx, idx]);

            let mut min_t = f32::INFINITY;
            let mut hit_idx = None;

            for (i, _) in scene.triangles.iter().enumerate() {
                let triangle = Triangle {
                    scene: &scene,
                    index: i,
                };

                if let Some(t) = ray.intersects(triangle) {
                    min_t = min_t.min(t);
                    hit_idx = Some(i);
                }
            }

            let color = if min_t.is_finite() {
                let hit_idx = hit_idx.unwrap();
                let triangle = Triangle {
                    scene: &scene,
                    index: hit_idx,
                };

                let normal = triangle.normal();
                let brightness = normal.dot(sun).max(0.0);
                let b = brightness;
                (b, b, b)
            } else {
                (0.0, 0.0, 0.0)
            };

            fb[pixel_idx] = color.0;
            fb[pixel_idx + 1] = color.1;
            fb[pixel_idx + 2] = color.2;
        }
    }

    // save_obj("debug.obj", &debug_scene)?;

    write_rgb_file(
        "output.exr",
        viewport_width as usize,
        viewport_height as usize,
        |x, y| {
            let idx = (y * viewport_width + x) * 3;
            (fb[idx], fb[idx + 1], fb[idx + 2])
        },
    )?;

    Ok(())
}
