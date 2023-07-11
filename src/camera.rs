use crate::matrix::Matrix4;

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

/** `cgmath`'s perspective matrix targets OpenGL clip space, which is:

* x: -1.0 to 1.0
* y: -1.0 to 1.0
* z: -1.0 to 1.0

WebGPU's clip space's `x` and `y` regions are the same, but `z` runs from
0.0 to 1.0 instead (reference: <https://www.w3.org/TR/webgpu/#coordinate-systems>).

Taken from <https://sotrh.github.io/learn-wgpu/beginner/tutorial6-uniforms/#a-perspective-camera>.
*/
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

impl Camera {
    /** Construct a transformation matrix that takes points in the camera's
    [viewing frustum](https://en.wikipedia.org/wiki/Viewing_frustum) to WGPU
    [clip space coordinates](https://www.w3.org/TR/webgpu/#clip-space-coordinates).
     */
    pub fn clip_coordinates_matrix(&self) -> Matrix4 {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let perspective =
            cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.near, self.far);

        Matrix4::from(OPENGL_TO_WGPU_MATRIX * perspective * view)
    }
}
