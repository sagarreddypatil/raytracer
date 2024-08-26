use crate::TVec3;
use nalgebra::{Matrix4, UnitQuaternion, Vector3};

pub const UP: TVec3 = Vector3::new(0.0, 0.0, 1.0);

/// Make a perspective projection matrix.
/// * `fov` - Field of view in radians
/// * `aspect` - Aspect ratio
pub fn perspective(fov: f32, aspect: f32) -> Matrix4<f32> {
    let focal_length = 1.0 / (fov / 2.0).tan();

    #[rustfmt::skip]
    let out = Matrix4::new(
        focal_length / aspect, 0.0,          0.0, 0.0,
        0.0,                   focal_length, 0.0, 0.0,
        0.0,                   0.0,          1.0, 1.0,
        0.0,                   0.0,         -1.0, 0.0,
    );

    out
}

pub struct Camera {
    projection: Matrix4<f32>,
    inv_projection: Matrix4<f32>,
    extrinsic: Matrix4<f32>,
}

impl Camera {
    pub fn extrinsic_matrix(&mut self) -> Matrix4<f32> {
        self.extrinsic
    }

    pub fn inverse_projection_matrix(&mut self) -> Matrix4<f32> {
        self.inv_projection
    }

    pub fn new(position: TVec3, rotation: UnitQuaternion<f32>, projection: Matrix4<f32>) -> Self {
        let extrinsic =
            Matrix4::from(rotation.to_rotation_matrix()) * Matrix4::new_translation(&-position);

        Self {
            projection,
            inv_projection: projection.try_inverse().unwrap(),
            extrinsic,
        }
    }
}
