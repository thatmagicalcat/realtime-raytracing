use std::time::Duration;

use eframe::egui;

pub struct Renderer {
    image_data: Vec<u32>,
    aspect_ratio: f32,
    pub render_time: Duration,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            image_data: vec![],
            aspect_ratio: 1.0,
            render_time: Duration::ZERO,
        }
    }

    pub fn render(&mut self, rect: egui::Rect) {
        let clock = std::time::Instant::now();

        let w = rect.max.x - rect.min.x;
        let h = rect.max.y - rect.min.y;

        self.aspect_ratio = w / h;

        self.image_data.clear();
        self.image_data.reserve((w * h) as _);

        for y in 0..h as usize {
            for x in 0..w as usize {
                let mut coord = glam::vec2(x as f32 / w, y as f32 / h) * 2.0 - 1.0;
                coord.x *= self.aspect_ratio;

                let color = self.render_pixel(coord);
                self.image_data.push(utils::convert_to_rgba(color));
            }
        }

        self.render_time = clock.elapsed();
    }

    #[inline]
    fn render_pixel(&self, coord: glam::Vec2) -> glam::Vec4 {
        let sphere_center = glam::Vec3::ZERO;
        let ray_origin = glam::vec3(0.0, 0.0, 1.0);
        let ray_direction = glam::vec3(coord.x, coord.y, -1.0);
        let radius = 0.5;
        let light_direction = glam::vec3(-1.0, -1.0, -1.0);

        let a = ray_direction.dot(ray_direction);
        let b = 2.0 * ray_origin.dot(ray_direction);
        let c = ray_origin.dot(ray_origin) - radius * radius;

        // b^2 - 4ac
        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return glam::vec4(0.0, 0.0, 0.0, 1.0);
        }

        let d_sqrt = discriminant.sqrt();

        let t = [
            (-b - d_sqrt) / (2.0 * a), // closest
            (-b + d_sqrt) / (2.0 * a),
        ];

        let h0 = ray_origin + t[0] * ray_direction;

        let normal = (h0 - sphere_center).normalize();
        let light_intensity = normal.dot(-light_direction).max(0.0) / light_direction.length();
        glam::vec4(light_intensity, 0.53 * light_intensity, 0.0, 1.0)
    }

    pub fn get_texture_data(&self) -> &[u32] {
        &self.image_data
    }
}

mod utils {
    pub fn convert_to_rgba(color: impl Into<glam::Vec4>) -> u32 {
        let color = color.into();
        ((color.w * 255.0) as u32) << 24
            | ((color.z * 255.0) as u32) << 16
            | ((color.y * 255.0) as u32) << 8
            | ((color.x * 255.0) as u32)
    }
}
