use nalgebra::Projective3;

use crate::{geom::Transform, Matrix4f, Projective, Vector3d};

pub const UP: Vector3d = Vector3d::new(0.0, 0.0, 1.0);

/// Make a perspective projection matrix.
/// * `fov` - Field of view in radians
/// * `aspect` - Aspect ratio
pub fn perspective(fov: f32, aspect: f32) -> Projective {
    let focal_length = 1.0 / (fov / 2.0).tan();

    #[rustfmt::skip]
    let out = Matrix4f::new(
        focal_length / aspect, 0.0,          0.0, 0.0,
        0.0,                   focal_length, 0.0, 0.0,
        0.0,                   0.0,          1.0, 1.0,
        0.0,                   0.0,         -1.0, 0.0,
    );

    Projective::from_matrix_unchecked(out)
}

pub struct Camera {
    pub width: usize,
    pub height: usize,

    pub transform: Transform,
    pub projection: Projective,
    pub inv_projection: Projective,
}

impl Camera {
    pub fn new(width: usize, height: usize, transform: Transform, projection: Projective) -> Self {
        Self {
            width,
            height,

            transform,

            projection,
            inv_projection: projection.inverse(),
        }
    }
}
