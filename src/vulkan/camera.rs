use nalgebra::{Matrix4, Vector4, Point3, Vector3};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CameraUniform {
    pub view_inverse: Matrix4<f32>,
    pub proj_inverse: Matrix4<f32>,
    pub position: Vector4<f32>,
}

pub struct Camera {
    pub position: Point3<f32>,
    pub forward: Vector3<f32>,
    pub up: Vector3<f32>,
    pub right: Vector3<f32>,
    
    pub aspect: f32,
    pub fov: f32,
    
    pub yaw: f32,
    pub pitch: f32,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        let mut cam = Self {
            position: Point3::new(32.0, 32.0, -10.0),
            forward: Vector3::zeros(),
            up: Vector3::y(),
            right: Vector3::zeros(),
            aspect,
            fov: 70.0_f32.to_radians(),
            yaw: -90.0_f32.to_radians(),
            pitch: 0.0,
        };
        cam.update_vectors();
        cam
    }

    pub fn update_aspect(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn input_rotate(&mut self, delta_x: f32, delta_y: f32, sensitivity: f32) {
        self.yaw += delta_x * sensitivity;
        self.pitch -= delta_y * sensitivity;

        let limit = 89.0_f32.to_radians();
        self.pitch = self.pitch.clamp(-limit, limit);

        self.update_vectors();
    }

    pub fn input_move(&mut self, direction: Vector3<f32>, speed: f32) {
        self.position += self.forward * direction.z * speed;
        self.position += self.right * direction.x * speed;
        self.position += Vector3::y() * direction.y * speed;
    }

    fn update_vectors(&mut self) {
        let forward = Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos()
        ).normalize();

        self.forward = forward;
        self.right = self.forward.cross(&Vector3::y()).normalize();
        self.up = self.right.cross(&self.forward).normalize();
    }

    pub fn get_uniform(&self) -> CameraUniform {
        let target = self.position + self.forward;
        let view = Matrix4::look_at_rh(&self.position, &target, &self.up);
        let mut proj = Matrix4::new_perspective(self.aspect, self.fov, 0.1, 1000.0);
        proj[(1, 1)] *= -1.0;

        CameraUniform {
            view_inverse: view.try_inverse().unwrap(),
            proj_inverse: proj.try_inverse().unwrap(),
            position: Vector4::new(self.position.x, self.position.y, self.position.z, 0.0),
        }
    }
}
