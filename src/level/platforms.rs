use raylib::prelude::*;

use crate::physics::collision::AABB;

pub struct Platform {
    pub aabb: AABB,
    pub color: Color,
}

impl Platform {
    pub fn new(min: Vector3, max: Vector3, color: Color) -> Self {
        Self {
            aabb: AABB::new(min, max),
            color,
        }
    }
}
