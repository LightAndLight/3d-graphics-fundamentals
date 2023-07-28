use cgmath::Transform;

use crate::{point::Point3, vector::Vec3};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Matrix4 {
    value: [[f32; 4]; 4],
}

impl From<cgmath::Matrix4<f32>> for Matrix4 {
    fn from(value: cgmath::Matrix4<f32>) -> Self {
        Matrix4 {
            value: value.into(),
        }
    }
}

impl From<Matrix4> for cgmath::Matrix4<f32> {
    fn from(value: Matrix4) -> Self {
        value.value.into()
    }
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
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

impl Matrix4 {
    pub fn perspective(fovy: f32, aspect: f32, near: f32, far: f32) -> Self {
        let perspective = cgmath::perspective(cgmath::Deg(fovy), aspect, near, far);
        Self::from(OPENGL_TO_WGPU_MATRIX * perspective)
    }

    pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let ortho = cgmath::ortho(left, right, bottom, top, near, far);
        Self::from(OPENGL_TO_WGPU_MATRIX * ortho)
    }

    pub fn look_at(eye: Point3, target: Point3, up: Vec3) -> Self {
        Self::from(cgmath::Matrix4::look_at_rh(
            eye.into(),
            target.into(),
            up.into(),
        ))
    }

    pub fn look_to(eye: Point3, dir: Vec3, up: Vec3) -> Self {
        Self::from(cgmath::Matrix4::look_to_rh(
            eye.into(),
            dir.into(),
            up.into(),
        ))
    }

    pub fn inverse(&self) -> Self {
        Self::from(
            cgmath::Matrix4::from(self.value)
                .inverse_transform()
                .unwrap(),
        )
    }
}

impl std::ops::Mul for Matrix4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::from(cgmath::Matrix4::<f32>::from(self) * cgmath::Matrix4::<f32>::from(rhs))
    }
}
