use nalgebra::{Affine3, Matrix3, Matrix4, Point3, Projective3, UnitQuaternion, Vector2, Vector3};

pub type Ray = bvh::ray::Ray<f32, 3>;

pub type Vector2f = Vector2<f32>;
pub type Vector3f = Vector3<f32>;

pub type Vector3d = Vector3<f64>;

pub type Point3f = Point3<f32>;
pub type Point3d = Point3<f64>;
pub type Quaternion = UnitQuaternion<f64>;

pub type Matrix3f = Matrix3<f32>;

pub type Matrix4f = Matrix4<f32>;
pub type Matrix4d = Matrix4<f64>;

pub type Affine = Affine3<f64>;
pub type Projective = Projective3<f32>;