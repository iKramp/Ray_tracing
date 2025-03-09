#![no_std]

#[allow(unused_imports)] //actually used for .sqrt because we don't allow std
use spirv_std::num_traits::Float;

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    padding: f64,
}

impl Vector3d {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vector3d {
            x,
            y,
            z,
            padding: 0.0,
        }
    }

    pub fn dot(self, rhs: Vector3d) -> f64 {
        rhs.x * self.x + rhs.y * self.y + rhs.z * self.z
    }

    pub fn norm2(self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn cross(self, rhs: Vector3d) -> Vector3d {
        Vector3d::new(
            rhs.z * self.y - rhs.y * self.z,
            rhs.x * self.z - rhs.z * self.x,
            rhs.y * self.x - rhs.x * self.y,
        )
    }

    pub fn normalize(&mut self) {
        let len = self.norm2().sqrt();
        if len == 0.0 {
            return;
        }
        self.x /= len;
        self.y /= len;
        self.z /= len;
    }

    pub fn normalized(self) -> Self {
        let mut result = self;
        result.normalize();
        result
    }
}

impl Default for Vector3d {
    fn default() -> Self {
        Vector3d::new(0.0, 0.0, 0.0)
    }
}

use core::iter::Sum;
use core::ops::{Add, Div, Mul, Neg, Sub};
impl Add<Vector3d> for Vector3d {
    type Output = Vector3d;
    fn add(self, rhs: Vector3d) -> Self::Output {
        Vector3d::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub<Vector3d> for Vector3d {
    type Output = Vector3d;
    fn sub(self, rhs: Vector3d) -> Self::Output {
        Vector3d::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Sum<Vector3d> for Vector3d {
    fn sum<I: Iterator<Item = Vector3d>>(mut iter: I) -> Vector3d {
        if let Some(first) = iter.next() {
            iter.fold(first, |a, b| a + b)
        } else {
            // There has got to be a more elegant way to do this, but
            // if so I don't see it.
            let x: Option<f64> = None;
            let zero_x: f64 = x.into_iter().sum();
            let y: Option<f64> = None;
            let zero_y: f64 = y.into_iter().sum();
            let z: Option<f64> = None;
            let zero_z: f64 = z.into_iter().sum();
            Vector3d::new(zero_x, zero_y, zero_z)
        }
    }
}

impl<'a> Sum<&'a Vector3d> for Vector3d {
    fn sum<I: Iterator<Item = &'a Vector3d>>(iter: I) -> Vector3d {
        iter.cloned().sum()
    }
}

impl Neg for Vector3d {
    type Output = Vector3d;
    fn neg(self) -> Self::Output {
        Vector3d::new(-self.x, -self.y, -self.z)
    }
}

impl<S: Clone> Mul<S> for Vector3d
where
    f64: Mul<S, Output = f64>,
{
    type Output = Vector3d;
    fn mul(self, rhs: S) -> Self::Output {
        Vector3d::new(self.x * rhs.clone(), self.y * rhs.clone(), self.z * rhs)
    }
}

impl<S: Clone> Div<S> for Vector3d
where
    f64: Div<S, Output = f64>,
{
    type Output = Vector3d;
    fn div(self, rhs: S) -> Self::Output {
        Vector3d::new(self.x / rhs.clone(), self.y / rhs.clone(), self.z / rhs)
    }
}
