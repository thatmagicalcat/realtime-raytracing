use std::time::Duration;

use eframe::egui;

use crate::camera::Camera;
use crate::ray::Ray;
use crate::world::World;

const BLACK: glam::Vec4 = glam::vec4(0.0, 0.0, 0.0, 1.0);

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

    pub fn render(&mut self, rect: egui::Rect, camera: &Camera, world: &World) {
        let clock = std::time::Instant::now();

        let w = rect.max.x - rect.min.x;
        let h = rect.max.y - rect.min.y;

        self.aspect_ratio = w / h;

        self.image_data.clear();
        self.image_data.reserve((w * h) as _);

        let ray_origin = camera.get_position();
        let mut ray = Ray {
            origin: *ray_origin,
            direction: glam::Vec3::ZERO,
        };

        for y in 0..h as usize {
            for x in 0..w as usize {
                ray.direction = camera.get_ray_directions()[x + y * w as usize];
                let color = self.trace_ray(&ray, world);

                self.image_data.push(utils::convert_to_rgba(&color));
            }
        }

        self.render_time = clock.elapsed();
    }

    #[inline]
    fn trace_ray(&self, ray: &Ray, world: &World) -> glam::Vec4 {
        if world.objects.is_empty() {
            return BLACK;
        }

        let Some((t, sphere)) = world
            .objects
            .iter()
            .filter_map(|sphere| {
                let origin = ray.origin - sphere.position;
                let a = ray.direction.dot(ray.direction);
                let b = 2.0 * origin.dot(ray.direction);
                let c = origin.dot(origin) - sphere.radius * sphere.radius;

                // b^2 - 4ac
                let discriminant = b * b - 4.0 * a * c;
                if discriminant < 0.0 {
                    return None;
                }

                let d_sqrt = discriminant.sqrt();
                let t = (-b - d_sqrt) / (2.0 * a);
                Some((t, sphere))
            })
            .min_by(|(t1, _), (t2, _)| t1.total_cmp(t2))
        else {
            return BLACK;
        };

        let light_direction = glam::vec3(-1.0, -1.0, -1.0);

        let origin = ray.origin - sphere.position;
        let h0 = origin + t * ray.direction;

        let normal = h0.normalize();
        let light_intensity = normal.dot(-light_direction).max(0.0) / light_direction.length();

        glam::Vec4::from((sphere.albedo * light_intensity, 1.0))
    }

    pub fn get_texture_data(&self) -> &[u32] {
        &self.image_data
    }
}

mod utils {
    pub fn convert_to_rgba(color: &glam::Vec4) -> u32 {
        ((color.w * 255.0) as u32) << 24
            | ((color.z * 255.0) as u32) << 16
            | ((color.y * 255.0) as u32) << 8
            | ((color.x * 255.0) as u32)
    }
}
