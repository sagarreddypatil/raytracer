mod types;
mod scene;
mod texture;
mod camera;
mod geom;
mod objfile;
mod render;

use scene::Scene;
use types::*;

use std::f32::consts::PI;

use anyhow::Result;
use camera::{perspective, Camera, UP};
use exr::prelude::*;
use geom::{BvhScene, Transform};
use nalgebra::{DMatrix, Matrix, Point3, UnitQuaternion, Vector2, Vector3, Vector4};

fn main() {
    if let Err(e) = real_main() {
        eprintln!("Error: {}", e);
    }
}

fn real_main() -> Result<()> {
    let object = objfile::load_obj("detailed-monkey.obj")?;

    println!(
        "Loaded object with {} vertices and {} triangles",
        object.mesh.vertices.len(),
        object.mesh.triangles.len()
    );

    let hdri = exr::image::read::read()
        .no_deep_data()
        .largest_resolution_level()
        .all_channels()
        .all_layers()
        .all_attributes()
        .on_progress(|_| {})
        .from_file("hdri.exr")?;

    let hdri_layers = hdri.layer_data;
    let hdri = &hdri_layers[0];

    let hdri_width = hdri.size.x();
    let hdri_height = hdri.size.y();

    let hdri_channel = |name: &str| {
        hdri.channel_data
            .list
            .iter()
            .filter(|c| c.name == *name)
            .next()
            .unwrap()
    };

    let hdri_r = hdri_channel("R").sample_data.values_as_f32();
    let hdri_g = hdri_channel("G").sample_data.values_as_f32();
    let hdri_b = hdri_channel("B").sample_data.values_as_f32();

    let hdri_gray = DMatrix::from_iterator(
        hdri_width,
        hdri_height,
        hdri_r
            .zip(hdri_g)
            .zip(hdri_b)
            .map(|((r, g), b)| 0.2126 * r + 0.7152 * g + 0.0722 * b),
    );

    println!("Loaded HDRI with resolution {}x{}", hdri_width, hdri_height);

    let viewport_width = 1920;
    let viewport_height = 1080;
    let aspect = viewport_width as f32 / viewport_height as f32;

    let camera_pos = Point3::new(0.0, -5.0, 0.0);
    let camera_dir = Quaternion::look_at_rh(&-camera_pos.coords, &UP);
    let fov = 120.0 * PI / 180.0;

    let camera = Camera::new(
        viewport_width,
        viewport_height,
        Transform::new(camera_pos, camera_dir, Vector3::new(1.0, 1.0, 1.0)),
        perspective(fov, aspect),
    );

    let mut scene = Scene::new(camera, vec![object], hdri_gray);

    println!("Starting render");
    scene.build_bvh();

    let fb = render::sample_once(&scene);

    write_rgb_file(
        "output.exr",
        viewport_width as usize,
        viewport_height as usize,
        |x, y| {
            let b = fb[(x, y)];
            (b, b, b)
        },
    )?;

    Ok(())
}
