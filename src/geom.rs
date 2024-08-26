use crate::camera::Camera;
use cgmath::{InnerSpace, Matrix4, Vector3};

pub struct Ray {
    pub origin: Vector3<f32>,
    pub direction: Vector3<f32>,
}

pub struct Triangle<'a> {
    pub scene: &'a Scene,
    pub index: usize,
}

impl Triangle<'_> {
    pub fn abc(&self) -> (Vector3<f32>, Vector3<f32>, Vector3<f32>) {
        let indices = self.scene.triangles[self.index];

        let a = self.scene.vertices[indices[0] as usize];
        let b = self.scene.vertices[indices[1] as usize];
        let c = self.scene.vertices[indices[2] as usize];

        // (a, b, c)
        (c, b, a)
    }

    pub fn normal(&self) -> Vector3<f32> {
        let (a, b, c) = self.abc();

        (b - a).cross(c - a).normalize()
    }
}

// from wikipedia
fn moller_trumbore_intersection (origin: Vector3<f32>, direction: Vector3<f32>, triangle: Triangle) -> Option<f32> {
    let (a, b, c) = triangle.abc();

	let e1 = b - a;
	let e2 = c - a;

	let ray_cross_e2 = direction.cross(e2);
	let det = e1.dot(ray_cross_e2);

	if det > -f32::EPSILON && det < f32::EPSILON {
		return None; // This ray is parallel to this triangle.
	}

	let inv_det = 1.0 / det;
	let s = origin - a;
	let u = inv_det * s.dot(ray_cross_e2);
	if u < 0.0 || u > 1.0 {
		return None;
	}

	let s_cross_e1 = s.cross(e1);
	let v = inv_det * direction.dot(s_cross_e1);
	if v < 0.0 || u + v > 1.0 {
		return None;
	}
	// At this stage we can compute t to find out where the intersection point is on the line.
	let t = inv_det * e2.dot(s_cross_e1);

	if t > f32::EPSILON { // ray intersection
		return Some(t);
	}
	else { // This means that there is a line intersection but not a ray intersection.
		return None;
	}
}

impl Ray {
    pub fn intersects<'a>(&self, triangle: Triangle<'a>) -> Option<f32> {
        moller_trumbore_intersection(self.origin, self.direction, triangle)
    }
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub vertices: Vec<Vector3<f32>>,
    pub triangles: Vec<[u32; 3]>,
    // materials: Vec<Material>,
}

impl Scene {
    pub fn transform(&self, matrix: &Matrix4<f32>) -> Scene {
        let mut vertices = Vec::new();
        for vertex in &self.vertices {
            vertices.push((matrix * vertex.extend(1.0)).truncate());
        }

        Scene {
            vertices,
            triangles: self.triangles.clone(),
        }
    }
}
