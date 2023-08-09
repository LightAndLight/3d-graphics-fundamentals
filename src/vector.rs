use cgmath::InnerSpace;

use crate::point::Point3;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub const X: Self = Vec3 {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };

    pub const Y: Self = Vec3 {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };

    pub const Z: Self = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };

    pub fn cross(self, rhs: Self) -> Self {
        cgmath::Vector3::<f32>::from(self)
            .cross(cgmath::Vector3::<f32>::from(rhs))
            .into()
    }

    pub fn dot(self, rhs: Self) -> f32 {
        cgmath::Vector3::<f32>::from(self).dot(cgmath::Vector3::<f32>::from(rhs))
    }

    pub fn length(self) -> f32 {
        cgmath::Vector3::<f32>::from(self).magnitude()
    }

    pub fn normalize(self) -> Self {
        let length = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        Vec3 {
            x: self.x / length,
            y: self.y / length,
            z: self.z / length,
        }
    }
}

impl From<Vec3> for cgmath::Vector3<f32> {
    fn from(value: Vec3) -> Self {
        cgmath::Vector3 {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<cgmath::Vector3<f32>> for Vec3 {
    fn from(value: cgmath::Vector3<f32>) -> Self {
        Vec3 {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl std::ops::Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl std::ops::AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl std::ops::Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl From<Vec2> for [f32; 2] {
    fn from(value: Vec2) -> Self {
        [value.x, value.y]
    }
}

impl From<Point3> for Vec3 {
    fn from(value: Point3) -> Self {
        Vec3 {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}
