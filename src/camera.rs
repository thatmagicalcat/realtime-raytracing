use glam::Vec4Swizzles;

use crate::{KeyboardState, PointerState};

pub struct Camera {
    vfov: f32,
    near_plane: f32,
    far_plane: f32,

    projection: glam::Mat4,
    view: glam::Mat4,
    inverse_projection: glam::Mat4,
    inverse_view: glam::Mat4,

    position: glam::Vec3,
    forward_direction: glam::Vec3,

    viewport_height: u32,
    viewport_width: u32,

    last_mouse_position: glam::Vec2,

    // cached
    ray_directions: Vec<glam::Vec3>,
}

impl Camera {
    pub fn new(vfov: f32, near_plane: f32, far_plane: f32) -> Self {
        Self {
            vfov,
            near_plane,
            far_plane,

            projection: glam::Mat4::IDENTITY,
            view: glam::Mat4::IDENTITY,
            inverse_projection: glam::Mat4::IDENTITY,
            inverse_view: glam::Mat4::IDENTITY,

            position: glam::vec3(0.0, 0.0, 3.0),
            forward_direction: glam::vec3(0.0, 0.0, -1.0),

            ray_directions: vec![],

            viewport_height: 0,
            viewport_width: 0,

            last_mouse_position: glam::Vec2::ZERO,
        }
    }

    pub fn update(&mut self, dt: f32, pointer_state: PointerState, keyboard_state: KeyboardState) {
        let Some(mouse_pos) = pointer_state.pos else {
            return;
        };

        if !pointer_state.secondary_down {
            self.last_mouse_position = mouse_pos;
            return;
        }

        let mouse_delta = (mouse_pos - self.last_mouse_position) * 0.002;
        self.last_mouse_position = mouse_pos;

        let up_direction = glam::Vec3::Y;
        let speed = 5.0;
        let right_direction = self.forward_direction.cross(up_direction);

        let mut moved = false;
        let mut f = |c: bool, v: glam::Vec3| {
            if c {
                self.position += v * speed * dt;
                moved = true;
            }
        };

        // translation
        f(keyboard_state.w, self.forward_direction);
        f(keyboard_state.s, -self.forward_direction);
        f(keyboard_state.a, -right_direction);
        f(keyboard_state.d, right_direction);
        f(keyboard_state.q, -up_direction);
        f(keyboard_state.e, up_direction);

        // rotation
        if mouse_delta.x != 0.0 || mouse_delta.y != 0.0 {
            let pitch_delta = mouse_delta.y * self.get_rotation_speed();
            let yaw_delta = mouse_delta.x * self.get_rotation_speed();

            let q = (glam::Quat::from_axis_angle(right_direction, -pitch_delta)
                * glam::Quat::from_axis_angle(glam::Vec3::Y, -yaw_delta))
            .normalize();

            self.forward_direction = q * self.forward_direction;

            moved = true;
        }

        if moved {
            self.recalculate_view();
            self.recalculate_raydirections();
        }
    }

    pub const fn get_rotation_speed(&self) -> f32 {
        0.3
    }

    pub fn resize(&mut self, new_height: u32, new_width: u32) {
        if self.viewport_height == new_height && self.viewport_width == new_width {
            return;
        }

        self.viewport_height = new_height;
        self.viewport_width = new_width;

        self.recalculate_projection();
        self.recalculate_raydirections();
    }

    pub fn recalculate_view(&mut self) {
        self.view = glam::Mat4::look_at_rh(
            self.position,
            self.position + self.forward_direction,
            glam::Vec3::Y,
        );
        self.inverse_view = self.view.inverse();
    }

    pub fn recalculate_raydirections(&mut self) {
        self.ray_directions.clear();
        self.ray_directions
            .reserve(self.viewport_height as usize * self.viewport_width as usize);

        for y in 0..self.viewport_height {
            for x in 0..self.viewport_width {
                let coord = glam::vec2(
                    x as f32 / self.viewport_width as f32,
                    y as f32 / self.viewport_height as f32,
                ) * 2.0
                    - 1.0;

                let target = self.inverse_projection * glam::vec4(coord.x, coord.y, 1.0, 1.0);

                let t = (target.xyz() / target.w).normalize();
                let ray_direction = (self.inverse_view * glam::vec4(t.x, t.y, t.z, 0.0)).xyz();
                self.ray_directions.push(ray_direction);
            }
        }
    }

    pub fn recalculate_projection(&mut self) {
        self.projection = glam::Mat4::perspective_rh(
            self.vfov,
            self.viewport_width as f32 / self.viewport_height as f32,
            self.near_plane,
            self.far_plane,
        );

        self.inverse_projection = self.projection.inverse();
    }

    pub fn get_projection(&self) -> &glam::Mat4 {
        &self.projection
    }

    pub fn get_inverse_projection(&self) -> &glam::Mat4 {
        &self.inverse_projection
    }

    pub fn get_view(&self) -> &glam::Mat4 {
        &self.view
    }

    pub fn get_inverse_view(&self) -> &glam::Mat4 {
        &self.inverse_view
    }

    pub fn get_position(&self) -> &glam::Vec3 {
        &self.position
    }

    pub fn get_direction(&self) -> &glam::Vec3 {
        &self.forward_direction
    }

    pub fn get_ray_directions(&self) -> &[glam::Vec3] {
        &self.ray_directions
    }
}
