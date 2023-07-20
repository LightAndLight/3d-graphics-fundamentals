use crate::{matrix::Matrix4, point::Point3};

pub struct Camera {
    /// The camera's position.
    pub eye: cgmath::Point3<f32>,

    /// The point at which the camera is looking.
    pub target: cgmath::Point3<f32>,

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
        let view = Matrix4::look_at(self.eye.into(), self.target.into(), self.up.into());
        let perspective = Matrix4::perspective(self.fovy, self.aspect, self.near, self.far);
        perspective * view
    }

    pub fn to_uniform(&self) -> CameraUniform {
        CameraUniform {
            eye: self.eye.into(),
            _padding: 0,
            view_proj: self.clip_coordinates_matrix(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub eye: Point3,
    /**
    See:

    * <https://sotrh.github.io/learn-wgpu/showcase/alignment/>
    * <https://sotrh.github.io/learn-wgpu/intermediate/tutorial10-lighting/#the-blinn-phong-model>
    */
    pub _padding: u32,
    pub view_proj: Matrix4,
}
