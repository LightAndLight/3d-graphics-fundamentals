use crate::{cuboid::Cuboid, matrix::Matrix4, point::Point3};

#[derive(Debug, Clone, Copy)]
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

    pub fn as_cuboid(&self) -> Cuboid {
        let near_top_left = Point3 {
            x: self.min.x,
            y: self.max.y,
            z: self.max.z,
        };
        let near_top_right = Point3 {
            x: self.max.x,
            y: self.max.y,
            z: self.max.z,
        };
        let near_bottom_left = Point3 {
            x: self.min.x,
            y: self.min.y,
            z: self.max.z,
        };
        let near_bottom_right = Point3 {
            x: self.max.x,
            y: self.min.y,
            z: self.max.z,
        };

        let far_top_left = Point3 {
            x: self.min.x,
            y: self.max.y,
            z: self.min.z,
        };
        let far_top_right = Point3 {
            x: self.max.x,
            y: self.max.y,
            z: self.min.z,
        };
        let far_bottom_left = Point3 {
            x: self.min.x,
            y: self.min.y,
            z: self.min.z,
        };
        let far_bottom_right = Point3 {
            x: self.max.x,
            y: self.min.y,
            z: self.min.z,
        };

        Cuboid {
            near_top_left,
            near_top_right,
            near_bottom_left,
            near_bottom_right,
            far_top_left,
            far_top_right,
            far_bottom_left,
            far_bottom_right,
        }
    }

    pub fn center(&self) -> Point3 {
        (self.min + self.max) / 2.0
    }

    pub fn transform(&self, matrix: Matrix4) -> Self {
        let value = matrix * self.as_cuboid();
        value.aabb()
    }
}
