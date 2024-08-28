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
    pub width: usize,
    pub height: usize,

    position: TVec3,
    rotation: UnitQuaternion<f32>,

    projection: Matrix4<f32>,
    inv_projection: Matrix4<f32>,

    extrinsic: Matrix4<f32>,
    inv_extrinsic: Matrix4<f32>,
}

impl Camera {
    pub fn extrinsic_matrix(&self) -> Matrix4<f32> {
        self.extrinsic
    }

    pub fn inverse_extrinsic_matrix(&self) -> Matrix4<f32> {
        self.inv_extrinsic
    }

    pub fn projection_matrix(&self) -> Matrix4<f32> {
        self.projection
    }

    pub fn inverse_projection_matrix(&self) -> Matrix4<f32> {
        self.inv_projection
    }

    pub fn new(width: usize, height: usize, position: TVec3, rotation: UnitQuaternion<f32>, projection: Matrix4<f32>) -> Self {
        let extrinsic =
            Matrix4::from(rotation.to_rotation_matrix()) * Matrix4::new_translation(&-position);

        Self {
            width,
            height,

            position,
            rotation,

            projection,
            inv_projection: projection.try_inverse().unwrap(),

            extrinsic,
            inv_extrinsic: extrinsic.try_inverse().unwrap(),
        }
    }

    pub fn vector_world_to_camera(&self, vec: TVec3) -> TVec3 {
        self.rotation.inverse() * vec
    }
}
