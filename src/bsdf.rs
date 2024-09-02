use crate::{rng::rand_direction, Vector3f};

pub const UP: Vector3f = Vector3f::new(0.0, 0.0, 1.0);

pub trait BSDF {
    fn value(&self, wavelength: f32, incedent: Vector3f, reflected: Vector3f) -> f32;
    fn sample(&self, wavelength: f32, reflected: Vector3f) -> Vector3f;
    fn pdf(&self, wavelength: f32, incedent: Vector3f, reflected: Vector3f) -> f32;
}

pub struct Lambertian {
    pub albedo: f32,
}

impl BSDF for Lambertian {
    fn value(&self, _wavelength: f32, _incedent: Vector3f, _reflected: Vector3f) -> f32 {
        self.albedo
    }

    fn sample(&self, _wavelength: f32, _reflected: Vector3f) -> Vector3f {
        rand_direction() + UP
    }

    fn pdf(&self, _wavelength: f32, incedent: Vector3f, _reflected: Vector3f) -> f32 {
        incedent.dot(&UP)
    }
}

pub struct Glossy {
}

impl BSDF for Glossy {
    fn value(&self, _wavelength: f32, incedent: Vector3f, reflected: Vector3f) -> f32 {
        0.0
    }

    fn sample(&self, _wavelength: f32, reflected: Vector3f) -> Vector3f {
        reflected - 2.0 * reflected.dot(&UP) * UP
    }

    fn pdf(&self, _wavelength: f32, incedent: Vector3f, reflected: Vector3f) -> f32 {
        0.0
    }
}