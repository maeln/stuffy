use cgmath::prelude::*;
use cgmath::{perspective, Deg, Matrix4, Point3, Vector3};

pub enum Direction {
    FORWARD,
    BACKWARD,
    LEFT,
    RIGHT,
    UP,
    DOWN,
}

impl PartialEq for Direction {
    fn eq(&self, other: &Direction) -> bool {
        self == other
    }
}

#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    pub forward: Vector3<f32>,
    pub world_up: Vector3<f32>,
    pub right: Vector3<f32>,
    pub up: Vector3<f32>,

    angle_h: f32,
    angle_v: f32,
}

impl Camera {
    pub fn new(position: Point3<f32>, forward: Vector3<f32>, world_up: Vector3<f32>) -> Camera {
        let right = forward.cross(world_up).normalize();
        let up = right.cross(forward).normalize();

        Camera {
            position,
            forward,
            world_up,
            right,
            up,
            angle_h: 0.0,
            angle_v: 0.0,
        }
    }

    pub fn view(&self) -> Matrix4<f32> {
        Matrix4::look_at(self.position, self.position + self.forward, self.up)
    }

    pub fn move_cam(&mut self, dir: &Direction, dt: f32) {
        let vel = 5.0 * dt;
        match dir {
            &Direction::FORWARD => self.position = self.position + self.forward * vel,
            &Direction::BACKWARD => self.position = self.position - self.forward * vel,
            &Direction::LEFT => self.position = self.position - self.right * vel,
            &Direction::RIGHT => self.position = self.position + self.right * vel,
            &Direction::UP => self.position = self.position + self.up * vel,
            &Direction::DOWN => self.position = self.position - self.up * vel,
        }
    }

    pub fn move_target(&mut self, dx: f32, dy: f32, dt: f32) {
        let vel = 0.01;
        self.angle_h = self.angle_h + dx * vel;
        self.angle_v = self.angle_v + dy * vel;

        let vcos = f32::cos(self.angle_v);
        self.forward = Vector3::new(
            vcos * f32::sin(self.angle_h),
            f32::sin(self.angle_v),
            vcos * f32::cos(self.angle_h),
        );

        self.right = self.forward.cross(self.world_up).normalize();
        self.up = self.right.cross(self.forward).normalize();
    }
}
