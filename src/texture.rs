use std::f32::consts::PI;

use nalgebra::DMatrix;

use crate::{TVec2, TVec3};

// pub struct Texture {
//     pub data: DMatrix<f32>
// }

pub trait Texture {
    fn sample_linear(&self, uv: TVec2) -> f32;
    fn sample_nearest(&self, uv: TVec2) -> f32;
}

fn idx_float(mat: &DMatrix<f32>, row: f32, col: f32) -> f32 {
    let row = (row as usize).min(mat.nrows() - 1);
    let col = (col as usize).min(mat.ncols() - 1);

    mat[(row, col)]
}

fn bound_uv(uv: TVec2) -> TVec2 {
    assert!(uv.x >= 0.0 && uv.x <= 1.0);
    assert!(uv.y >= 0.0 && uv.y <= 1.0);

    // let x = uv.x.min(1.0).max(0.0);
    // let y = uv.y.min(1.0).max(0.0);

    let x = uv.x;
    let y = uv.y;

    TVec2::new(x, y)
}

impl Texture for DMatrix<f32> {
    fn sample_linear(&self, uv: TVec2) -> f32 {
        let uv = bound_uv(uv);

        let x = uv.x * self.nrows() as f32;
        let y = uv.y * self.ncols() as f32;

        let tl = idx_float(self, x, y);
        let tr = idx_float(self, x + 1.0, y);
        let bl = idx_float(self, x, y + 1.0);
        let br = idx_float(self, x + 1.0, y + 1.0);

        let x = x.fract();
        let y = y.fract();

        let top = tl * (1.0 - x) + tr * x;
        let bottom = bl * (1.0 - x) + br * x;

        top * (1.0 - y) + bottom * y
    }

    fn sample_nearest(&self, uv: TVec2) -> f32 {
        let uv = bound_uv(uv);

        let x = (uv.x * self.nrows() as f32).round();
        let y = (uv.y * self.ncols() as f32).round();

        idx_float(self, x, y)
    }
}

pub fn equirectangular(point: TVec3) -> TVec2 {
    assert!(point.norm() - 1.0 < 1e-6);

    let x = (point.x.atan2(point.y)) / (2.0 * PI) + 0.5;
    let y = point.z.acos() / PI;

    TVec2::new(x, y)
}
