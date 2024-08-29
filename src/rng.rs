use std::f32::consts::PI;

use crate::{geom::normalize, Vector2f, Vector3f};

static mut SEED: u32 = 0;

pub fn rand() -> u32 {
    unsafe {
        SEED = SEED.wrapping_mul(1664525).wrapping_add(1013904223);
        SEED
    }
}

pub fn rand_f32() -> f32 {
    let x: u32 = rand();
    x as f32 / u32::MAX as f32
}

pub fn rand_circle() -> Vector2f {
    let theta = 2.0 * PI * rand_f32();
    let rho = rand_f32().sqrt();

    Vector2f::new(rho * theta.cos(), rho * theta.sin())
}

pub fn rand_norm_f32() -> f32 {
    let theta = 2.0 * PI * rand_f32();
    let rho = (-2.0 * rand_f32().ln()).sqrt();

    rho * theta.cos()
}

pub fn rand_direction() -> Vector3f {
    let x = rand_norm_f32();
    let y = rand_norm_f32();
    let z = rand_norm_f32();

    // Vector3f::new(x, y, z).normalize()
    normalize(Vector3f::new(x, y, z))
}

pub fn rand_hemisphere(normal: Vector3f) -> Vector3f {
    let mut dir = rand_direction();
    while dir.norm() > 1.0 {
        dir = rand_direction();
    }

    if dir.dot(&normal) < 0.0 {
        -dir
    } else {
        dir
    }
}

// const MIN: i32 = -2147483648;
// const MAX: i32 = 2147483647;

// fn xorshift(value: i32) -> i32 {
//     // Xorshift*32
//     // Based on George Marsaglia's work: http://www.jstatsoft.org/v08/i14/paper
//     let mut value = value;
//     value ^= value << 13;
//     value ^= value >> 17;
//     value ^= value << 5;
//     value
// }

// fn next_int(seed: &mut i32) -> i32 {
//     let value = xorshift(*seed);
//     *seed = value;
//     value
// }

// fn next_float(seed: &mut i32) -> f32 {
//     let value = xorshift(*seed);
//     (value as f32 / 3141.592653).abs()
// }

// fn next_float_max(seed: &mut i32, max: f32) -> f32 {
//     next_float(seed) * max
// }

// static mut SEED: i32 = 0;

// pub fn rand_f32() -> f32 {
//     unsafe {
//         next_float(&mut SEED)
//     }
// }
