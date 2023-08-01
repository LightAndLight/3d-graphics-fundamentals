use std::ops::{Add, Sub};

use crate::vector::Vec3;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3 {
    pub const ZERO: Point3 = Point3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub fn with_w(self, w: f32) -> Point4 {
        Point4 {
            x: self.x,
            y: self.y,
            z: self.z,
            w,
        }
    }
}

impl Add for Point3 {
    type Output = Point3;

    fn add(self, rhs: Self) -> Self::Output {
        Point3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Point3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl From<cgmath::Point3<f32>> for Point3 {
    fn from(value: cgmath::Point3<f32>) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<Point3> for cgmath::Point3<f32> {
    fn from(value: Point3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<Point4> for Point3 {
    fn from(value: Point4) -> Self {
        Self {
            x: value.x / value.w,
            y: value.y / value.w,
            z: value.z / value.w,
        }
    }
}

pub struct Point4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl From<Point4> for cgmath::Vector4<f32> {
    fn from(value: Point4) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            w: value.w,
        }
    }
}

impl From<cgmath::Vector4<f32>> for Point4 {
    fn from(value: cgmath::Vector4<f32>) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            w: value.w,
        }
    }
}

impl std::ops::Add for Point4 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }
}
