use std::sync::Arc;
use std::sync::Mutex;

use crate::renderer::Renderer;
use crate::Program;

use eframe::egui;
use eframe::glow;

use eframe::CreationContext;
use glow::HasContext;

pub struct Application {
    vao: glow::VertexArray,
    texture_id: glow::NativeTexture,
    program: Arc<Program>,
    renderer: Arc<Mutex<Renderer>>,
    last_rect: egui::Rect,
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

        Self {
            texture_id,
            vao,
            program: Arc::new(program),
            renderer: Arc::new(Renderer::new().into()),
            last_rect: egui::Rect::ZERO,
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
        ctx.set_pixels_per_point(1.0);

        egui::Window::new("Info")
            .default_pos((10.0, 10.0))
            .collapsible(true)
            .resizable(true)
            .show(ctx, |ui| {
                ui.label(format!(
                    "Frame time: {:?}",
                    self.renderer.lock().unwrap().render_time
                ));
                ui.separator();
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let (rect, _response) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::drag());

                let old_rect = self.last_rect;
                self.last_rect = rect;

                let program = Arc::clone(&self.program);
                let vao = self.vao;
                let texture_id = self.texture_id;

                self.renderer.lock().unwrap().render(rect);
                let renderer = Arc::clone(&self.renderer);

                let callback = egui::PaintCallback {
                    rect,
                    callback: std::sync::Arc::new(eframe::egui_glow::CallbackFn::new(
                        move |_info, painter| {
                            let gl = painter.gl();

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
                        },
                    )),
                };

                ui.painter().add(callback);
            });
        });

        ctx.request_repaint();
    }
}
