use cgmath::Vector3;
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
};

use crate::geom::Scene;

pub fn load_obj(path: &str) -> anyhow::Result<Scene> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut vertices = Vec::new();
    let mut triangles = Vec::new();

    for line in reader.lines() {
        let line = line?;

        if line.starts_with('#') {
            continue;
        }

        let mut parts = line.split_whitespace();
        let next = parts.next();

        match next {
            Some("v") => {
                let x: f32 = parts.next().unwrap().parse()?;
                let y: f32 = parts.next().unwrap().parse()?;
                let z: f32 = parts.next().unwrap().parse()?;

                vertices.push(Vector3::new(x, y, z));
            }
            Some("f") => {
                // if it's quad, make two triangles
                if parts.clone().count() == 4 {
                    let mut indices = [0; 4];
                    for i in 0..4 {
                        let part = parts.next().unwrap();
                        let mut parts = part.split('/');

                        let index: u32 = parts.next().unwrap().parse()?;
                        indices[i] = index - 1;
                    }
                    triangles.push([indices[0], indices[1], indices[2]]);
                    triangles.push([indices[0], indices[2], indices[3]]);
                } else {
                    let mut triangle = [0; 3];
                    for i in 0..3 {
                        let part = parts.next().unwrap();
                        let mut parts = part.split('/');

                        let index: u32 = parts.next().unwrap().parse()?;
                        triangle[i] = index - 1;
                    }
                    triangles.push(triangle);
                }
            }
            _ => {}
        }
    }

    Ok(Scene {
        vertices,
        triangles,
    })
}

pub fn save_obj(path: &str, scene: &Scene) -> anyhow::Result<()> {
    let mut file = File::create(path)?;

    for vertex in &scene.vertices {
        writeln!(file, "v {} {} {}", vertex.x, vertex.y, vertex.z)?;
    }

    for triangle in &scene.triangles {
        writeln!(
            file,
            "f {} {} {}",
            triangle[0] + 1,
            triangle[1] + 1,
            triangle[2] + 1
        )?;
    }

    Ok(())
}
