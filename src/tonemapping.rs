use nalgebra::DMatrix;

pub fn tonemap(image: &DMatrix<f32>) -> Vec<u32> {
    image
        .iter()
        .map(|&x| {
            let b = (x * 255.0) as u32;
            (b << 16) | (b << 8) | b
        })
        .collect()
}
