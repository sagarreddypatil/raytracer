use cgmath::{Matrix4, Quaternion, Vector3};

pub const UP: Vector3<f32> = Vector3::new(0.0, 0.0, 1.0);

pub struct Camera {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,

    pub projection: Matrix4<f32>,

    memoized_pos_rot: Option<(Vector3<f32>, Quaternion<f32>)>,
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
            self.memoized_extrinsic = Some(Matrix4::from(self.rotation) * Matrix4::from_translation(-self.position));
        }

        self.memoized_extrinsic.unwrap()
    }

    pub fn new(
        position: Vector3<f32>,
        rotation: Quaternion<f32>,
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
