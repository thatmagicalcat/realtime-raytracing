#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub position: glam::Vec3,
    pub radius: f32,
    pub albedo: glam::Vec3,
}

impl Sphere {
    pub fn new(position: glam::Vec3, radius: f32, albedo: glam::Vec3) -> Self {
        Self {
            position,
            radius,
            albedo,
        }
    }
}

#[derive(Debug, Clone)]
pub struct World {
    pub objects: Vec<Sphere>,
}
