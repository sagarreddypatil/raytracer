use nalgebra::DMatrix;

pub fn aces_filmic(x: f32) -> f32 {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;

    (x * (a * x + b)) / (x * (c * x + d) + e)
}

pub fn tonemap(brightness: f32) -> f32 {
    aces_filmic(brightness)
}
