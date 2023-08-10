use crate::{matrix::Matrix4, point::Point3};

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
