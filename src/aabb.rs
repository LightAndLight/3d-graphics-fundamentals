use crate::{matrix::Matrix4, point::Point3};

#[derive(Debug)]
pub struct Aabb {
    pub min: Point3,
    pub max: Point3,
}

impl Aabb {
    pub const EMPTY: Self = Aabb {
        min: Point3::ZERO,
        max: Point3::ZERO,
    };

    pub fn union(self, other: Self) -> Self {
        Self {
            min: Point3 {
                x: self.min.x.min(other.min.x),
                y: self.min.y.min(other.min.y),
                z: self.min.z.min(other.min.z),
            },
            max: Point3 {
                x: self.max.x.max(other.max.x),
                y: self.max.y.max(other.max.y),
                z: self.max.z.max(other.max.z),
            },
        }
    }

    pub fn point(value: Point3) -> Self {
        Self {
            min: value,
            max: value,
        }
    }

    pub fn valid(&self) -> bool {
        self.min.x <= self.max.x && self.min.y <= self.max.y && self.min.z <= self.max.z
    }
}

impl std::ops::Mul<Aabb> for Matrix4 {
    type Output = Aabb;

    fn mul(self, rhs: Aabb) -> Self::Output {
        Aabb {
            min: Point3::from(self * rhs.min.with_w(1.0)),
            max: Point3::from(self * rhs.max.with_w(1.0)),
        }
    }
}

impl std::ops::Mul<&Aabb> for Matrix4 {
    type Output = Aabb;

    fn mul(self, rhs: &Aabb) -> Self::Output {
        Aabb {
            min: Point3::from(self * rhs.min.with_w(1.0)),
            max: Point3::from(self * rhs.max.with_w(1.0)),
        }
    }
}
