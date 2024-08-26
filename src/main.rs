mod camera;
mod geom;
mod obj;
mod render;

use std::f32::consts::PI;

use anyhow::Result;
use camera::{perspective, Camera, UP};
use exr::prelude::*;
use geom::{BVHScene, TRay};
use nalgebra::{Point3, UnitQuaternion, Vector3, Vector4};
use obj::save_obj;

use rayon::prelude::*;

pub type TVec3 = Vector3<f32>;

fn main() {
    if let Err(e) = real_main() {
        eprintln!("Error: {}", e);
    }
}

fn real_main() -> Result<()> {
    let scene = obj::load_obj("balls.obj")?;
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

    let viewport_width = 1920;
    let viewport_height = 1080;
    let aspect = viewport_width as f32 / viewport_height as f32;

    let camera_pos = Vector3::new(2.0, -3.0, 2.0);
    let camera_dir = UnitQuaternion::look_at_rh(&-camera_pos, &UP);
    let fov = 80.0 * PI / 180.0;

    let mut camera = Camera::new(
        viewport_width,
        viewport_height,
        camera_pos,
        camera_dir,
        perspective(fov, aspect),
    );

    // let mut debug_scene = scene.clone();

    let scene = scene.transform(&camera.extrinsic_matrix());
    let sun = Vector3::new(1.0, 0.0, 0.5).normalize();

    // let mut debug_scene = scene.clone();

    let scene: BVHScene = scene.into();

    println!("Starting render");

    let n_pixels = viewport_width * viewport_height;
    let fb = render::sample(&scene, &camera, sun);

    // save_obj("debug.obj", &debug_scene)?;

    write_rgb_file(
        "output.exr",
        viewport_width as usize,
        viewport_height as usize,
        |x, y| {
            let idx = y * viewport_width + x;
            let (r, g, b, a) = fb[idx];
            (r, g, b)
        },
    )?;

    Ok(())
}
