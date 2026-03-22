use raylib::prelude::*;

use crate::level::platforms::Platform;

pub struct Level {
    pub platforms: Vec<Platform>,
    pub spawn_points: Vec<Vector3>,
}

impl Level {
    pub fn test_level() -> Self {
        let platforms = vec![
            // Floor
            Platform::wall(
                Vector3::new(-15.0, -1.0, -2.0),
                Vector3::new(15.0, 0.0, 2.0),
            ),
            // Left wall
            Platform::wall(
                Vector3::new(-15.0, 0.0, -2.0),
                Vector3::new(-14.0, 12.0, 2.0),
            ),
            // Right wall
            Platform::wall(
                Vector3::new(14.0, 0.0, -2.0),
                Vector3::new(15.0, 12.0, 2.0),
            ),
            // Left mid platform
            Platform::platform(
                Vector3::new(-8.0, 3.5, -2.0),
                Vector3::new(-3.0, 4.0, 2.0),
            ),
            // Right mid platform
            Platform::platform(
                Vector3::new(3.0, 3.5, -2.0),
                Vector3::new(8.0, 4.0, 2.0),
            ),
            // Center high platform
            Platform::platform(
                Vector3::new(-2.5, 7.0, -2.0),
                Vector3::new(2.5, 7.5, 2.0),
            ),
        ];

        Self {
            platforms,
            spawn_points: vec![
                Vector3::new(-6.0, 0.0, 0.0),
                Vector3::new(6.0, 0.0, 0.0),
                Vector3::new(-10.0, 0.0, 0.0),
                Vector3::new(10.0, 0.0, 0.0),
            ],
        }
    }
}
