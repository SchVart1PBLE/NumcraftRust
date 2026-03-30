use libm::tanf;
use nalgebra::Vector3;

use crate::{camera::Camera, physic::BoundingBox};

struct Plane {
    normal: Vector3<f32>,
    pos: Vector3<f32>,
}

pub struct Frustum {
    top_plane: Plane,
    bottom_plane: Plane,

    right_plane: Plane,
    left_plane: Plane,

    far_plane: Plane,
    near_plane: Plane,
}

impl Frustum {
    pub fn new(camera: &Camera, aspect: f32, fov_y: f32, z_near: f32, z_far: f32) -> Self {
        let half_vside = z_far * tanf(fov_y * 0.5);
        let half_hside = half_vside * aspect;
        let front = camera.get_forward_vector();
        let up = camera.get_up_vector();
        let right = camera.get_right_vector();
        let front_mult_far = z_far * front;

        let cam_pos = *camera.get_pos();

        Frustum {
            near_plane: Plane {
                pos: cam_pos + z_near * front,
                normal: front,
            },
            far_plane: Plane {
                pos: cam_pos + front_mult_far,
                normal: -front,
            },
            right_plane: Plane {
                pos: cam_pos,
                normal: up.cross(&(front_mult_far - right * half_hside)),
            },
            left_plane: Plane {
                pos: cam_pos,
                normal: (front_mult_far + right * half_hside).cross(&up),
            },
            top_plane: Plane {
                pos: cam_pos,
                normal: (front_mult_far - up * half_vside).cross(&right),
            },
            bottom_plane: Plane {
                pos: cam_pos,
                normal: right.cross(&(front_mult_far + up * half_vside)),
            },
        }
    }

    pub fn is_aabb_in_frustum(&self, min: Vector3<f32>, max: Vector3<f32>) -> bool {
        for plane in [
            &self.far_plane,
            &self.near_plane,
            &self.right_plane,
            &self.left_plane,
            &self.top_plane,
            &self.bottom_plane,
        ] {
            // Pick the "positive vertex" — the corner of the AABB most in the direction of the plane normal
            let p = Vector3::new(
                if plane.normal.x >= 0.0 { max.x } else { min.x },
                if plane.normal.y >= 0.0 { max.y } else { min.y },
                if plane.normal.z >= 0.0 { max.z } else { min.z },
            );
            if (p - plane.pos).dot(&plane.normal) < 0.0 {
                return false;
            }
        }
        true
    }

    pub fn is_bbox_in_frustum(&self, bbox: &BoundingBox) -> bool {
        let min = bbox.offset;
        let max = bbox.offset + bbox.size;
        self.is_aabb_in_frustum(min, max)
    }
}
