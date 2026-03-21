use raylib::prelude::*;

use crate::level::platforms::Platform;

pub struct Level {
    pub platforms: Vec<Platform>,
    pub spawn_points: Vec<Vector3>,
}

impl Level {
    pub fn test_level() -> Self {
        let wall_color = Color::new(60, 60, 70, 255);
        let plat_color = Color::new(45, 45, 55, 255);

        let platforms = vec![
            // Floor
            Platform::new(
                Vector3::new(-15.0, -1.0, -2.0),
                Vector3::new(15.0, 0.0, 2.0),
                wall_color,
            ),
            // Left wall
            Platform::new(
                Vector3::new(-15.0, 0.0, -2.0),
                Vector3::new(-14.0, 12.0, 2.0),
                wall_color,
            ),
            // Right wall
            Platform::new(
                Vector3::new(14.0, 0.0, -2.0),
                Vector3::new(15.0, 12.0, 2.0),
                wall_color,
            ),
            // Left mid platform
            Platform::new(
                Vector3::new(-8.0, 3.5, -2.0),
                Vector3::new(-3.0, 4.0, 2.0),
                plat_color,
            ),
            // Right mid platform
            Platform::new(
                Vector3::new(3.0, 3.5, -2.0),
                Vector3::new(8.0, 4.0, 2.0),
                plat_color,
            ),
            // Center high platform
            Platform::new(
                Vector3::new(-2.5, 7.0, -2.0),
                Vector3::new(2.5, 7.5, 2.0),
                plat_color,
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
