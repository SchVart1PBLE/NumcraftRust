use core::f32::{self, consts::PI};

use libm::{cosf, sinf};
use nalgebra::{Matrix4, UnitQuaternion, Vector3};

use crate::{
    constants::{player::ROTATION_SPEED, rendering::FOV},
    nadk,
    input_manager::InputManager,
    settings::Settings,
};

pub struct Camera {
    pos: Vector3<f32>,
    rotation: Vector3<f32>,
    has_moved: bool,
    fov: f32,
    // Cached trig values — recomputed only when rotation changes
    sin_x: f32,
    cos_x: f32,
    sin_y: f32,
    cos_y: f32,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            pos: Vector3::new(0., 0., 0.),
            rotation: Vector3::new(0., 0., 0.),
            has_moved: true,
            fov: FOV,
            sin_x: 0.0,
            cos_x: 1.0,
            sin_y: 0.0,
            cos_y: 1.0,
        }
    }

    #[inline]
    fn update_trig_cache(&mut self) {
        self.sin_x = sinf(self.rotation.x);
        self.cos_x = cosf(self.rotation.x);
        self.sin_y = sinf(self.rotation.y);
        self.cos_y = cosf(self.rotation.y);
    }

    pub fn get_fov(&self) -> f32 {
        self.fov
    }

    pub fn set_fov(&mut self, degrees: f32) {
        self.fov = degrees * PI / 180.0;
    }

    pub fn update(&mut self, delta: f32, input_manager: &InputManager, settings: &Settings) {
        let mut rotated = false;
        // Rotation
        if (input_manager.is_keydown(nadk::keyboard::Key::Right) && !settings.reverse_controls)
            || (input_manager.is_keydown(nadk::keyboard::Key::Power) && settings.reverse_controls)
        {
            self.rotation.y -= delta * ROTATION_SPEED;
            rotated = true;
        }
        if (input_manager.is_keydown(nadk::keyboard::Key::Left) && !settings.reverse_controls)
            || (input_manager.is_keydown(nadk::keyboard::Key::Imaginary) && settings.reverse_controls)
        {
            self.rotation.y += delta * ROTATION_SPEED;
            rotated = true;
        }
        if (input_manager.is_keydown(nadk::keyboard::Key::Up) && !settings.reverse_controls)
            || (input_manager.is_keydown(nadk::keyboard::Key::Toolbox) && settings.reverse_controls)
        {
            self.rotation.x -= delta * ROTATION_SPEED;
            if self.rotation.x <= -PI / 2.0 + 0.0001 {
                self.rotation.x = -PI / 2.0 + 0.0001
            }
            rotated = true;
        }
        if (input_manager.is_keydown(nadk::keyboard::Key::Down) && !settings.reverse_controls)
            || (input_manager.is_keydown(nadk::keyboard::Key::Comma) && settings.reverse_controls)
        {
            self.rotation.x += delta * ROTATION_SPEED;
            if self.rotation.x >= PI / 2.0 - 0.0001 {
                self.rotation.x = PI / 2.0 - 0.0001
            }
            rotated = true;
        }
        if rotated {
            self.update_trig_cache();
        }
    }

    pub fn update_pos(&mut self, position: Vector3<f32>) {
        self.has_moved = self.pos != position;
        self.pos = position; // Updated from player
    }

    #[inline]
    pub fn get_forward_vector(&self) -> Vector3<f32> {
        Vector3::new(
            self.cos_x * self.sin_y,
            -self.sin_x,
            self.cos_x * self.cos_y,
        )
    }

    #[inline]
    pub fn get_right_vector(&self) -> Vector3<f32> {
        Vector3::new(self.cos_y, 0.0, -self.sin_y)
    }

    #[inline]
    pub fn get_up_vector(&self) -> Vector3<f32> {
        Vector3::new(
            self.sin_x * self.sin_y,
            self.cos_x,
            self.sin_x * self.cos_y,
        )
    }

    pub fn get_rotation_matrix(&self) -> Matrix4<f32> {
        let yaw = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), self.rotation.y);
        let pitch = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), self.rotation.x);
        let orientation = yaw * pitch;
        orientation.to_homogeneous()
    }

    pub fn get_pos(&self) -> &Vector3<f32> {
        &self.pos
    }

    pub fn set_rotation(&mut self, rot: Vector3<f32>) {
        self.rotation = rot;
        self.update_trig_cache();
    }

    pub fn get_has_moved(&self) -> bool {
        self.has_moved
    }

    pub fn get_rotation(&self) -> &Vector3<f32> {
        &self.rotation
    }
}
