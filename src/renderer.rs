use std::time::Duration;

use eframe::egui;

pub struct Renderer {
    image_data: Vec<u32>,
    pub render_time: Duration,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            image_data: vec![],
            render_time: Duration::ZERO,
        }
    }

    pub fn render(&mut self, rect: egui::Rect) {
        let clock = std::time::Instant::now();

        let w = rect.max.x - rect.min.x;
        let h = rect.max.y - rect.min.y;

        self.image_data.clear();
        self.image_data.reserve((w * h) as _);

        for y in 0..h as usize {
            for x in 0..w as usize {
                self.image_data.push(self.render_pixel(glam::vec2(
                    x as f32 / w, y as f32 / h,
                ) * 2.0 - 1.0));
            }
        }

        self.render_time = clock.elapsed();
    }

    #[inline]
    fn render_pixel(&self, coord: glam::Vec2) -> u32 {
        let r = (coord.x * 255.0) as u32;
        let g = (coord.y * 255.0) as u32;

        let ray_origin = glam::vec3(0.0, 0.0, 2.0);
        let ray_direction = glam::vec3(coord.x, coord.y, -1.0);
        let radius = 0.5;

        let a = ray_direction.dot(ray_direction);
        let b = 2.0 * ray_origin.dot(ray_direction);
        let c = ray_origin.dot(ray_origin) - radius * radius;

        // b^2 - 4ac
        let discriminant = b * b - 4.0 * a * c;
        if discriminant >= 0.0 {
            return 0xFFFF00FF;
        }

        0xFF000000 | g << 8 | r
    }

    pub fn get_texture_data(&self) -> &[u32] {
        &self.image_data
    }
}
