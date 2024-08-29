mod camera;
mod geom;
mod objfile;
mod render;
mod rng;
mod scene;
mod texture;
mod tonemapping;
mod types;

use exr::prelude::{write_rgb_file, ReadChannels, ReadLayers};
use indicatif::{ProgressBar, ProgressIterator};
use scene::Scene;
use tonemapping::tonemap;
use types::*;

use std::{f64::consts::PI, time::Instant};

use anyhow::Result;
use camera::{perspective, Camera, UP};
use geom::Transform;
use nalgebra::{DMatrix, Vector3};

fn main() {
    if let Err(e) = real_main() {
        eprintln!("Error: {}", e);
    }
}

fn rad(deg: f64) -> f64 {
    deg * PI / 180.0
}

fn camera_transform(pos: Point3d) -> Transform {
    let look_at = -pos.coords;

    Transform::new(
        pos,
        Quaternion::look_at_rh(&look_at, &UP),
        // Quaternion::from_euler_angles(0.0, rad(45.0), 0.0),
        Vector3::new(1.0, 1.0, 1.0),
    )
}

fn real_main() -> Result<()> {
    let mut object = objfile::load_obj("smooth-monkey.obj")?;
    object.transform = Transform::new(
        Point3d::new(0.0, 0.0, 0.0),
        Quaternion::from_euler_angles(0.0, 0.0, PI / 2.0),
        Vector3::new(1.0, 1.0, 1.0),
    );
    println!("{:?}", object);

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

    let gray = hdri_r
        .zip(hdri_g)
        .zip(hdri_b)
        .map(|((r, g), b)| 0.2126 * r + 0.7152 * g + 0.0722 * b);

    let hdri_gray = DMatrix::from_iterator(hdri_width, hdri_height, gray);

    // let hdri_width = 2048;
    // let hdri_height = 1024;
    // let hdri_gray = DMatrix::zeros(2048, 1024);

    println!("Loaded HDRI with resolution {}x{}", hdri_width, hdri_height);

    let viewport_width = 1280;
    let viewport_height = 720;
    let aspect = viewport_width as f32 / viewport_height as f32;

    let fov = rad(50.0);

    let camera = Camera::new(
        viewport_width,
        viewport_height,
        camera_transform(Point3d::new(2.5, 0.5, 1.0)),
        perspective(fov as f32, aspect),
    );

    let mut scene = Scene::new(camera, vec![object], hdri_gray);

    println!("Starting render");

    scene.build_bvh();
    let samples = 32;
    let bar = ProgressBar::new(samples as u64);

    let mut fb: DMatrix<_> = DMatrix::zeros(viewport_width, viewport_height);
    let mut sample_times = Vec::with_capacity(samples as usize);

    for _ in (0..samples).progress_with(bar) {
        let time_start = Instant::now();
        fb += render::sample_once(&scene);

        let elapsed = time_start.elapsed();
        sample_times.push(elapsed);
    }

    // use last 8 samples to estimate time
    let time_per_sample = sample_times.iter().rev().take(8).sum::<std::time::Duration>() / 8;
    println!("Time per sample: {:?}", time_per_sample);

    write_rgb_file(
        // "output.exr",
        &format!("output.exr"),
        viewport_width as usize,
        viewport_height as usize,
        |x, y| {
            let b = fb[(x, y)];
            let b = b / samples as f32;

            let b = tonemap(b);
            (b, b, b)
        },
    )?;

    Ok(())
}
