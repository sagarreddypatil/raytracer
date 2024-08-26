use cgmath::{Vector3, Matrix4};

pub struct Scene {
    pub vertices: Vec<Vector3<f32>>,
    pub triangles: Vec<[u32; 3]>,
    // materials: Vec<Material>,
}