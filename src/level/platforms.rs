use raylib::prelude::*;

use crate::physics::collision::AABB;

pub struct Platform {
    pub aabb: AABB,
    pub is_wall: bool,
}

impl Platform {
    pub fn wall(min: Vector3, max: Vector3) -> Self {
        Self {
            aabb: AABB::new(min, max),
            is_wall: true,
        }
    }

    pub fn platform(min: Vector3, max: Vector3) -> Self {
        Self {
            aabb: AABB::new(min, max),
            is_wall: false,
        }
    }
}
