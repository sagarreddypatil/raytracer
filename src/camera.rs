use nalgebra::{Matrix4, UnitQuaternion, Vector3};
use crate::TVec3;

pub const UP: TVec3 = Vector3::new(0.0, 0.0, 1.0);

pub struct Camera {
    pub position: TVec3,
    pub rotation: UnitQuaternion<f32>,

    pub projection: Matrix4<f32>,

    memoized_pos_rot: Option<(TVec3, UnitQuaternion<f32>)>,
    memoized_extrinsic: Option<Matrix4<f32>>,
}

/// Make a perspective projection matrix.
/// * `fov` - Field of view in radians
/// * `aspect` - Aspect ratio
pub fn perspective(fov: f32, aspect: f32) -> Matrix4<f32> {
    let S = 1.0 / (fov / 2.0).tan();

    #[rustfmt::skip]
    let out = Matrix4::new(
        S / aspect, 0.0,  0.0, 0.0,
        0.0,        S,    0.0, 0.0,
        0.0,        0.0,  1.0, 1.0,
        0.0,        0.0, -1.0, 0.0,
    );

    out
}

impl Camera {
    pub fn extrinsic_matrix(&mut self) -> Matrix4<f32> {
        if Some((self.position, self.rotation)) != self.memoized_pos_rot {
            self.memoized_pos_rot = Some((self.position, self.rotation));
            self.memoized_extrinsic = Some(Matrix4::from(self.rotation) * Matrix4::new_translation(&-self.position));
        }

        self.memoized_extrinsic.unwrap()
    }

    pub fn new(
        position: TVec3,
        rotation: UnitQuaternion<f32>,
        projection: Matrix4<f32>,
    ) -> Self {
        Self {
            position,
            rotation,
            projection,

            memoized_pos_rot: None,
            memoized_extrinsic: None,
        }
    }
}
