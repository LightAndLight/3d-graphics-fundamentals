use crate::{
    cuboid::Cuboid,
    matrix::Matrix4,
    point::{Point3, Point4},
};

const CLIP_NEAR_TOP_LEFT: Point4 = Point4 {
    x: -1.0,
    y: 1.0,
    z: 0.0,
    w: 1.0,
};

const CLIP_NEAR_TOP_RIGHT: Point4 = Point4 {
    x: 1.0,
    y: 1.0,
    z: 0.0,
    w: 1.0,
};

const CLIP_NEAR_BOTTOM_LEFT: Point4 = Point4 {
    x: -1.0,
    y: -1.0,
    z: 0.0,
    w: 1.0,
};

const CLIP_NEAR_BOTTOM_RIGHT: Point4 = Point4 {
    x: 1.0,
    y: -1.0,
    z: 0.0,
    w: 1.0,
};

const CLIP_FAR_TOP_LEFT: Point4 = Point4 {
    x: -1.0,
    y: 1.0,
    z: 1.0,
    w: 1.0,
};

const CLIP_FAR_TOP_RIGHT: Point4 = Point4 {
    x: 1.0,
    y: 1.0,
    z: 1.0,
    w: 1.0,
};

const CLIP_FAR_BOTTOM_LEFT: Point4 = Point4 {
    x: -1.0,
    y: -1.0,
    z: 1.0,
    w: 1.0,
};

const CLIP_FAR_BOTTOM_RIGHT: Point4 = Point4 {
    x: 1.0,
    y: -1.0,
    z: 1.0,
    w: 1.0,
};
pub struct Camera {
    /// The camera's position.
    pub eye: Point3,

    /// The direction in which the camera is looking.
    pub direction: cgmath::Vector3<f32>,

    /// The "up" direction relative to the camera.
    pub up: cgmath::Vector3<f32>,

    /// The ratio of viewport width to height.
    pub aspect: f32,

    /// The number of degrees of vertical field of view.
    pub fovy: f32,

    /// The near plane of the frustum. Mapped to Z = 0.0 in NDC: anything between
    /// `eye.z` and `eye.z + near` will be clipped.
    pub near: f32,

    /// The far plane of the frustum. Mapped to Z = 1.0 in NDC: anything further
    /// than `eye.z + far` will be clipped.
    pub far: f32,
}

impl Camera {
    /** Construct a transformation matrix that takes points in the camera's
    [viewing frustum](https://en.wikipedia.org/wiki/Viewing_frustum) to WGPU
    [clip space coordinates](https://www.w3.org/TR/webgpu/#clip-space-coordinates).
     */
    pub fn clip_coordinates_matrix(&self) -> Matrix4 {
        let view = Matrix4::look_to(self.eye, self.direction.into(), self.up.into());
        let perspective = Matrix4::perspective(self.fovy, self.aspect, self.near, self.far);
        perspective * view
    }

    pub fn to_uniform(&self) -> CameraUniform {
        let view_proj = self.clip_coordinates_matrix();
        CameraUniform {
            eye: self.eye,
            zfar: self.far,
            view_proj,
            view_proj_inv: view_proj.inverse(),
        }
    }

    pub fn frustum_world_space(&self) -> Cuboid {
        let clip_to_world = self.clip_coordinates_matrix().inverse();

        Cuboid {
            near_top_left: Point3::from(clip_to_world * CLIP_NEAR_TOP_LEFT) + self.eye,
            near_top_right: Point3::from(clip_to_world * CLIP_NEAR_TOP_RIGHT) + self.eye,
            near_bottom_left: Point3::from(clip_to_world * CLIP_NEAR_BOTTOM_LEFT) + self.eye,
            near_bottom_right: Point3::from(clip_to_world * CLIP_NEAR_BOTTOM_RIGHT) + self.eye,
            far_top_left: Point3::from(clip_to_world * CLIP_FAR_TOP_LEFT) + self.eye,
            far_top_right: Point3::from(clip_to_world * CLIP_FAR_TOP_RIGHT) + self.eye,
            far_bottom_left: Point3::from(clip_to_world * CLIP_FAR_BOTTOM_LEFT) + self.eye,
            far_bottom_right: Point3::from(clip_to_world * CLIP_FAR_BOTTOM_RIGHT) + self.eye,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub eye: Point3,
    pub zfar: f32,
    pub view_proj: Matrix4,
    pub view_proj_inv: Matrix4,
}
