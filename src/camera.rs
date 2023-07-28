use crate::{matrix::Matrix4, point::Point3};

pub struct Camera {
    /// The camera's position.
    pub eye: cgmath::Point3<f32>,

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
        let view = Matrix4::look_to(self.eye.into(), self.direction.into(), self.up.into());
        let perspective = Matrix4::perspective(self.fovy, self.aspect, self.near, self.far);
        perspective * view
    }

    pub fn to_uniform(&self) -> CameraUniform {
        let view_proj = self.clip_coordinates_matrix();
        CameraUniform {
            eye: self.eye.into(),
            zfar: self.far,
            view_proj,
            view_proj_inv: view_proj.inverse(),
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
