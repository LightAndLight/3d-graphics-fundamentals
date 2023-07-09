use std::ops::Add;

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
