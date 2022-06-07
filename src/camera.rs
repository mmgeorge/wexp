use bytemuck::{Zeroable, Pod};
use cgmath::{Point3, Vector3, Matrix4, perspective, Deg, SquareMatrix};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
  1.0, 0.0, 0.0, 0.0,
  0.0, 1.0, 0.0, 0.0,
  0.0, 0.0, 0.5, 0.0,
  0.0, 0.0, 0.5, 1.0,
);


pub const TEST_MAT: Matrix4<f32> = Matrix4::new(
  0.5, 0.0, 0.0, 0.0,
  0.0, 0.5, 0.0, 0.0,
  0.0, 0.0, 0.5, 0.0,
  0.0, 0.0, 0.5, 1.0,
);

pub struct Camera {
  eye: Point3<f32>,
  target: Point3<f32>,
  up: Vector3<f32>, 
  aspect: f32,
  fovy: f32,
  znear: f32,
  zfar: f32,
}

impl Camera {
  pub fn new(aspect: f32) -> Self {
    Self {
      eye: (0., 2., 2.).into(),
      target: (0., 0., 0.).into(),
      up: Vector3::unit_y(),
      aspect,
      fovy: 45.,
      znear: 0.1,
      zfar: 100.
    }
  }
  
  pub fn vp_mat(&self) -> Matrix4<f32> {
    let view = Matrix4::look_at_rh(self.eye, self.target, self.up);
    let proj = perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);

    // TEST_MAT
    OPENGL_TO_WGPU_MATRIX * proj * view
  }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
  vp_mat: [[f32; 4]; 4]
}

impl CameraUniform {
  pub fn new() -> Self {
    let vp_mat = Matrix4::identity().into(); 

    Self { vp_mat }
  }

  pub fn update(&mut self, camera: &Camera) {
    self.vp_mat = camera.vp_mat().into(); 
  }
}
