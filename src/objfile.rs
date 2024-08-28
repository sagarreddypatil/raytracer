use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
};

use crate::geom::{Mesh, Object, Transform};
use nalgebra::{Point3, Vector3};

// use crate::geom::Scene;

pub fn load_obj(path: &str) -> anyhow::Result<Object> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut vertices = Vec::new();
    let mut triangles = Vec::new();

    let mut normals = Vec::new();
    let mut normal_triangles = Vec::new();

    for line in reader.lines() {
        let line = line?;

        if line.starts_with('#') {
            continue;
        }

        let mut parts = line.split_terminator(' ');
        let next = parts.next();

        match next {
            Some("v") => {
                let x: f32 = parts.next().unwrap().parse()?;
                let y: f32 = parts.next().unwrap().parse()?;
                let z: f32 = parts.next().unwrap().parse()?;

                vertices.push(Point3::new(x, y, z));
            }
            Some("vn") => {
                let x: f32 = parts.next().unwrap().parse()?;
                let y: f32 = parts.next().unwrap().parse()?;
                let z: f32 = parts.next().unwrap().parse()?;

                normals.push(Vector3::new(x, y, z));
            }
            Some("f") => {
                let mut triangle = [0; 3];
                let mut normal_triangle = [0; 3];
                for i in 0..3 {
                    let part = parts.next().unwrap();
                    let parts: Vec<_> = part.split('/').collect();

                    let index: u32 = parts[0].parse()?;
                    triangle[i] = index - 1;

                    let normal_index: u32 = parts[2].parse()?;
                    normal_triangle[i] = normal_index - 1;
                }
                triangles.push(triangle);
                normal_triangles.push(normal_triangle);
            }
            _ => {}
        }
    }

    Ok(Object {
        transform: Transform::identity(),
        mesh: Mesh {
            vertices,
            triangles,
            normals,
            normal_triangles,
        },
    })
}

pub fn save_obj(path: &str, object: &Object) -> anyhow::Result<()> {
    let mut file = File::create(path)?;
    let mesh = &object.mesh;

    for vertex in &mesh.vertices {
        writeln!(file, "v {} {} {}", vertex.x, vertex.y, vertex.z)?;
    }

    for normal in &mesh.normals {
        writeln!(file, "vn {} {} {}", normal.x, normal.y, normal.z)?;
    }

    for (triangle, normal_triangle) in mesh.triangles.iter().zip(mesh.normal_triangles.iter()) {
        writeln!(
            file,
            "f {}//{} {}//{} {}//{}",
            triangle[0] + 1,
            normal_triangle[0] + 1,
            triangle[1] + 1,
            normal_triangle[1] + 1,
            triangle[2] + 1,
            normal_triangle[2] + 1
        )?;
    }

    Ok(())
}
