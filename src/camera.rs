use winit::window::Window;

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
        self.perspective_matrix() * self.view_matrix()
    }

    pub fn view_matrix(&self) -> Matrix4 {
        Matrix4::look_to(self.eye, self.direction.into(), self.up.into())
    }

    pub fn perspective_matrix(&self) -> Matrix4 {
        Matrix4::perspective(self.fovy, self.aspect, self.near, self.far)
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
            near_top_left: Point3::from(clip_to_world * CLIP_NEAR_TOP_LEFT),
            near_top_right: Point3::from(clip_to_world * CLIP_NEAR_TOP_RIGHT),
            near_bottom_left: Point3::from(clip_to_world * CLIP_NEAR_BOTTOM_LEFT),
            near_bottom_right: Point3::from(clip_to_world * CLIP_NEAR_BOTTOM_RIGHT),
            far_top_left: Point3::from(clip_to_world * CLIP_FAR_TOP_LEFT),
            far_top_right: Point3::from(clip_to_world * CLIP_FAR_TOP_RIGHT),
            far_bottom_left: Point3::from(clip_to_world * CLIP_FAR_BOTTOM_LEFT),
            far_bottom_right: Point3::from(clip_to_world * CLIP_FAR_BOTTOM_RIGHT),
        }
    }

    pub fn frustum_camera_space(&self) -> Cuboid {
        let clip_to_camera = self.perspective_matrix().inverse();

        Cuboid {
            near_top_left: Point3::from(clip_to_camera * CLIP_NEAR_TOP_LEFT),
            near_top_right: Point3::from(clip_to_camera * CLIP_NEAR_TOP_RIGHT),
            near_bottom_left: Point3::from(clip_to_camera * CLIP_NEAR_BOTTOM_LEFT),
            near_bottom_right: Point3::from(clip_to_camera * CLIP_NEAR_BOTTOM_RIGHT),
            far_top_left: Point3::from(clip_to_camera * CLIP_FAR_TOP_LEFT),
            far_top_right: Point3::from(clip_to_camera * CLIP_FAR_TOP_RIGHT),
            far_bottom_left: Point3::from(clip_to_camera * CLIP_FAR_BOTTOM_LEFT),
            far_bottom_right: Point3::from(clip_to_camera * CLIP_FAR_BOTTOM_RIGHT),
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

pub struct MouseLook {
    enabled: bool,
}

impl MouseLook {
    pub fn new(window: &Window, enabled: bool) -> Self {
        let mut this = Self { enabled };
        this.set(window, enabled);
        this
    }

    pub fn set(&mut self, window: &Window, value: bool) {
        window.set_cursor_visible(!value);
        self.enabled = value;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }
}
