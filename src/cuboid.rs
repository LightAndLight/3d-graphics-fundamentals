use crate::{aabb::Aabb, matrix::Matrix4, point::Point3};

pub struct Cuboid {
    pub near_top_left: Point3,
    pub near_top_right: Point3,
    pub near_bottom_left: Point3,
    pub near_bottom_right: Point3,
    pub far_top_left: Point3,
    pub far_top_right: Point3,
    pub far_bottom_left: Point3,
    pub far_bottom_right: Point3,
}

impl Cuboid {
    pub fn wireframe_mesh(&self) -> [(Point3, Point3); 12] {
        [
            // Near
            (self.near_top_left, self.near_top_right),
            (self.near_top_right, self.near_bottom_right),
            (self.near_bottom_right, self.near_bottom_left),
            (self.near_bottom_left, self.near_top_left),
            // Far
            (self.far_top_left, self.far_top_right),
            (self.far_top_right, self.far_bottom_right),
            (self.far_bottom_right, self.far_bottom_left),
            (self.far_bottom_left, self.far_top_left),
            // Between
            (self.near_top_left, self.far_top_left),
            (self.near_top_right, self.far_top_right),
            (self.near_bottom_right, self.far_bottom_right),
            (self.near_bottom_left, self.far_bottom_left),
        ]
    }

    pub fn center(&self) -> Point3 {
        (self.near_bottom_left
            + self.near_bottom_right
            + self.near_top_left
            + self.near_top_right
            + self.far_bottom_left
            + self.far_bottom_right
            + self.far_top_left
            + self.far_top_right)
            / 8.0
    }

    pub fn aabb(&self) -> Aabb {
        let vertices = [
            self.far_bottom_left,
            self.far_bottom_right,
            self.far_top_left,
            self.far_top_right,
            self.near_bottom_left,
            self.near_bottom_right,
            self.near_top_left,
            self.near_top_right,
        ];

        let (min_x, max_x, min_y, max_y, min_z, max_z) = vertices.into_iter().fold(
            (
                f32::INFINITY,
                f32::NEG_INFINITY,
                f32::INFINITY,
                f32::NEG_INFINITY,
                f32::INFINITY,
                f32::NEG_INFINITY,
            ),
            |(min_x, max_x, min_y, max_y, min_z, max_z), point| {
                (
                    min_x.min(point.x),
                    max_x.max(point.x),
                    min_y.min(point.y),
                    max_y.max(point.y),
                    min_z.min(point.z),
                    max_z.max(point.z),
                )
            },
        );

        Aabb {
            min: Point3 {
                x: min_x,
                y: min_y,
                z: min_z,
            },
            max: Point3 {
                x: max_x,
                y: max_y,
                z: max_z,
            },
        }
    }
}

impl std::ops::Mul<Cuboid> for Matrix4 {
    type Output = Cuboid;

    fn mul(self, rhs: Cuboid) -> Self::Output {
        Cuboid {
            near_top_left: Point3::from(self * rhs.near_top_left.with_w(1.0)),
            near_top_right: Point3::from(self * rhs.near_top_right.with_w(1.0)),
            near_bottom_left: Point3::from(self * rhs.near_bottom_left.with_w(1.0)),
            near_bottom_right: Point3::from(self * rhs.near_bottom_right.with_w(1.0)),
            far_top_left: Point3::from(self * rhs.far_top_left.with_w(1.0)),
            far_top_right: Point3::from(self * rhs.far_top_right.with_w(1.0)),
            far_bottom_left: Point3::from(self * rhs.far_bottom_left.with_w(1.0)),
            far_bottom_right: Point3::from(self * rhs.far_bottom_right.with_w(1.0)),
        }
    }
}
