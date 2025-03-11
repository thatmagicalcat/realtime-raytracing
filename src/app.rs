use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;

use crate::camera::Camera;
use crate::renderer::Renderer;
use crate::world::*;
use crate::Program;

use eframe::egui;
use eframe::glow;

use eframe::CreationContext;
use glow::HasContext;

pub struct Application {
    camera: Camera,
    vao: glow::VertexArray,
    texture_id: glow::NativeTexture,
    program: Arc<Program>,
    renderer: Arc<Mutex<Renderer>>,
    last_rect: egui::Rect,
    gpu_time: Arc<Mutex<Duration>>,
    clock: Instant,
    world: World,
}

impl Application {
    pub fn new(cc: &CreationContext) -> Self {
        let gl = Arc::clone(cc.gl.as_ref().unwrap());

        #[rustfmt::skip]
        let vertices: &[f32] = &[
            // position      texture
             1.0,  1.0,      1.0, 1.0, // top right
             1.0, -1.0,      1.0, 0.0, // bottom right
            -1.0, -1.0,      0.0, 0.0, // bottom left
            -1.0,  1.0,      0.0, 1.0, // top left
        ];

        #[rustfmt::skip]
        let indices: &[u32] = &[
            0, 1, 3,
            1, 2, 3,
        ];

        let vertices_slice: &[u8] = unsafe {
            std::slice::from_raw_parts(vertices.as_ptr() as _, std::mem::size_of_val(vertices))
        };

        let indices_slice: &[u8] = unsafe {
            std::slice::from_raw_parts(indices.as_ptr() as _, std::mem::size_of_val(indices))
        };

        let vao = unsafe { gl.create_vertex_array().unwrap() };
        let vbo = unsafe { gl.create_buffer().unwrap() };
        let ebo = unsafe { gl.create_buffer().unwrap() };

        let texture_id = unsafe { gl.create_texture().unwrap() };

        unsafe {
            gl.bind_vertex_array(Some(vao));

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, indices_slice, glow::STATIC_DRAW);

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_slice, glow::STATIC_DRAW);

            // position
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(
                0,
                2,
                glow::FLOAT,
                false,
                4 * std::mem::size_of::<f32>() as i32,
                0,
            );

            // texture coordinate
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(
                1,
                2,
                glow::FLOAT,
                false,
                4 * std::mem::size_of::<f32>() as i32,
                2 * std::mem::size_of::<f32>() as i32,
            );

            // unbind
            gl.bind_vertex_array(None);
        }

        // texture
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(texture_id));

            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );

            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );

            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );

            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );

            gl.bind_texture(glow::TEXTURE_2D, None);
        }

        let program = Program::from_str(
            Arc::clone(&gl),
            include_str!("shader.glsl"),
            "vertex",
            "fragment",
        )
        .unwrap();

        let objects = vec![
            Sphere {
                position: glam::vec3(0.0, 0.0, 0.0),
                radius: 0.5,
                albedo: glam::vec3(1.0, 0.0, 1.0),
            },
            Sphere {
                position: glam::vec3(-0.2, -0.3, -3.0),
                radius: 2.0,
                albedo: glam::vec3(1.0, 0.53, 0.0),
            },
        ];

        Self {
            camera: Camera::new(45.0_f32.to_radians(), 0.1, 100.0),
            clock: Instant::now(),
            texture_id,
            vao,
            program: Arc::new(program),
            renderer: Arc::new(Renderer::new().into()),
            last_rect: egui::Rect::ZERO,
            gpu_time: Arc::new(Duration::ZERO.into()),
            world: World { objects },
        }
    }

    unsafe fn update_texture(
        gl: &glow::Context,
        texture_id: glow::NativeTexture,
        image_vec: &[u32],
        old_rect: egui::Rect,
        rect: egui::Rect,
    ) {
        let data = std::slice::from_raw_parts(
            image_vec.as_ptr() as *const u8,
            std::mem::size_of_val(image_vec),
        );

        let w = rect.max.x - rect.min.x;
        let h = rect.max.y - rect.min.y;

        gl.bind_texture(glow::TEXTURE_2D, Some(texture_id));

        if old_rect == rect {
            gl.tex_sub_image_2d(
                glow::TEXTURE_2D,
                0,
                0,
                0,
                w as _,
                h as _,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(data)),
            );
        } else {
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                w as _,
                h as _,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(data)),
            );
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let dt = self.clock.elapsed().as_nanos() as f32 / 1e9;
        self.clock = Instant::now();

        let pointer_state = ctx.input(|i| crate::PointerState {
            pos: i
                .pointer
                .hover_pos()
                .map(|egui::Pos2 { x, y }| glam::vec2(x, y)),
            secondary_down: i.pointer.secondary_down(),
        });

        let keyboard_state = ctx.input(|i| crate::KeyboardState {
            w: i.keys_down.contains(&egui::Key::W),
            a: i.keys_down.contains(&egui::Key::A),
            s: i.keys_down.contains(&egui::Key::S),
            d: i.keys_down.contains(&egui::Key::D),
            q: i.keys_down.contains(&egui::Key::Q),
            e: i.keys_down.contains(&egui::Key::E),
        });

        self.camera.update(dt, pointer_state, keyboard_state);
        if pointer_state.secondary_down {
            ctx.set_cursor_icon(egui::CursorIcon::None);
        } else {
            ctx.set_cursor_icon(egui::CursorIcon::Default);
        }

        ctx.set_pixels_per_point(1.0);

        egui::Window::new("Info")
            .default_pos((10.0, 10.0))
            .collapsible(true)
            .resizable(true)
            .show(ctx, |ui| {
                let gpu_time = *self.gpu_time.lock().unwrap();
                let render_time = self.renderer.lock().unwrap().render_time;

                ui.label(format!("GPU time: {gpu_time:?}"));
                ui.label(format!("Render time: {render_time:?}",));
                ui.separator();
                ui.label(format!("Frame time: {:?}", gpu_time + render_time));
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let (rect, _response) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::drag());

                let old_rect = self.last_rect;

                if old_rect != rect {
                    self.camera.resize(
                        (rect.max.y - rect.min.y) as _,
                        (rect.max.x - rect.min.x) as _,
                    );
                }

                self.last_rect = rect;

                let program = Arc::clone(&self.program);
                let vao = self.vao;
                let texture_id = self.texture_id;
                let gpu_time = Arc::clone(&self.gpu_time);

                self.renderer
                    .lock()
                    .unwrap()
                    .render(rect, &self.camera, &self.world);
                let renderer = Arc::clone(&self.renderer);

                let callback = egui::PaintCallback {
                    rect,
                    callback: std::sync::Arc::new(eframe::egui_glow::CallbackFn::new(
                        move |_info, painter| {
                            let gl = painter.gl();

                            let clock = Instant::now();

                            unsafe {
                                Self::update_texture(
                                    gl,
                                    texture_id,
                                    renderer.lock().unwrap().get_texture_data(),
                                    old_rect,
                                    rect,
                                )
                            }

                            unsafe {
                                gl.clear(glow::COLOR_BUFFER_BIT);

                                program.use_program();

                                gl.active_texture(glow::TEXTURE0);
                                gl.bind_texture(glow::TEXTURE_2D, Some(texture_id));

                                gl.bind_vertex_array(Some(vao));
                                gl.draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_INT, 0);
                            }

                            *gpu_time.lock().unwrap() = clock.elapsed();
                        },
                    )),
                };

                ui.painter().add(callback);
            });
        });

        ctx.request_repaint();
    }
}
