mod camera;
mod geom;
mod objfile;
mod render;
mod scene;
mod texture;
mod types;

use indicatif::{ProgressBar, ProgressIterator};
use scene::Scene;
use types::*;

use std::f64::consts::PI;

use anyhow::Result;
use camera::{perspective, Camera, UP};
use exr::prelude::*;
use geom::Transform;
use nalgebra::{DMatrix, Point3, Vector3};

use rayon::prelude::*;

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
        Quaternion::identity(),
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

    let hdri_gray = DMatrix::from_iterator(
        hdri_width,
        hdri_height,
        gray,
    );

    println!("Loaded HDRI with resolution {}x{}", hdri_width, hdri_height);

    let viewport_width = 1920;
    let viewport_height = 1080;
    let aspect = viewport_width as f32 / viewport_height as f32;

    let fov = rad(50.0);

    let camera = Camera::new(
        viewport_width,
        viewport_height,
        camera_transform(Point3d::new(-2.0, -3.0, 1.0)),
        perspective(fov as f32, aspect),
    );

    let mut scene = Scene::new(camera, vec![object], hdri_gray);

    println!("Starting render");

    // for i in 1..2 {
    //     let wow = i as f64;
    //     let wow = (wow / 4.0) - 5.0 + 1e-4;
    //     let camera_transform = camera_transform(Point3d::new(0.0, wow, 2.0));
    //     scene.camera.transform = camera_transform;

    //     scene.build_bvh();
    //     let fb = render::sample_once(&scene);

    //     write_rgb_file(
    //         &format!("output_{}.exr", i),
    //         viewport_width,
    //         viewport_height,
    //         |x, y| {
    //             let b = fb[(x, y)];
    //             (b, b, b)
    //         },
    //     )?;
    // }

    scene.build_bvh();
    let samples = 16;
    let bar = ProgressBar::new(samples as u64);
    let fb: DMatrix<_> = (0..samples).progress_with(bar).map(|_| {
        render::sample_once(&scene)
    }).sum();

    let fb = fb / samples as f32;

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
