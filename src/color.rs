use crate::Vector3f;

pub type Color = Vector3f;

fn linear_rec709_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        let gamma = 2.4;
        1.055 * c.powf(1.0 / gamma) - 0.055
    }
}

fn filmic(t: f32) -> f32 {
    let a = 0.22;
    let b = 0.3;
    let c = 0.1;
    let d = 0.2;
    let e = 0.01;
    let f = 0.3;

    ((t * (a * t + c * b) + d * e) / (t * (a * t + b) + d * f)) - e / f
}

// pub fn tonemap(color: Color) -> Color {
//     let exposure = 0.0;
//     let color = color * 2.0f32.powf(exposure);

//     // apply filmic tone mapping
//     let color = Color::new(
//         filmic(color.x),
//         filmic(color.y),
//         filmic(color.z),
//     );

//     // apply transfer function
//     Color::new(
//         linear_rec709_to_srgb(color.x),
//         linear_rec709_to_srgb(color.y),
//         linear_rec709_to_srgb(color.z),
//     )
// }

pub fn tonemap(color: Color) -> Color {
    color
}