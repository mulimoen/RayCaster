use cgmath::Matrix4;
use glium::{
    texture::{Texture2d, Texture3d},
    uniform, IndexBuffer, Program, Surface, VertexBuffer,
};
use imgui_glium_renderer::Renderer;

mod cube;
mod raycast;
mod support;

fn main() {
    let events_loop = glium::glutin::event_loop::EventLoop::new();
    let builder = glium::glutin::window::WindowBuilder::new();
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_depth_buffer(24);
    let display = glium::Display::new(builder, context, &events_loop).unwrap();

    let cube_pos = VertexBuffer::new(&display, &cube::VERTICES).unwrap();

    let cube_models = {
        let data = cube::make_matrices(20); // 20x20x20 cubes
        VertexBuffer::new(&display, &data).unwrap()
    };

    let cube_ind = IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &cube::INDICES,
    )
    .unwrap();
    let cube_prog =
        Program::from_source(&display, cube::VERT_SHADER, cube::FRAG_SHADER, None).unwrap();

    let quad_pos = VertexBuffer::new(&display, &raycast::VERTICES).unwrap();
    let quad_ind = IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &raycast::INDICES,
    )
    .unwrap();
    let quad_prog =
        Program::from_source(&display, raycast::VERT_SHADER, raycast::FRAG_SHADER, None)
            .expect("Could not compile fragment shader");

    struct Textures {
        backface: Texture2d,
        frontface: Texture2d,
        noise: Texture2d,
    }

    let textures = Textures {
        backface: Texture2d::empty_with_format(
            &display,
            glium::texture::UncompressedFloatFormat::F32F32F32F32,
            glium::texture::MipmapsOption::NoMipmap,
            1024,
            1024,
        )
        .unwrap(),
        frontface: Texture2d::empty_with_format(
            &display,
            glium::texture::UncompressedFloatFormat::F32F32F32F32,
            glium::texture::MipmapsOption::NoMipmap,
            1024,
            1024,
        )
        .unwrap(),
        noise: {
            let random_bytes = include_bytes!("random.bin").to_vec();

            Texture2d::with_format(
                &display,
                glium::texture::RawImage2d {
                    data: std::borrow::Cow::Owned(random_bytes),
                    width: 1024,
                    height: 1024,
                    format: glium::texture::ClientFormat::U8,
                },
                glium::texture::UncompressedFloatFormat::U8,
                glium::texture::MipmapsOption::NoMipmap,
            )
            .unwrap()
        },
    };
    struct DepthBuffers {
        frontface: glium::framebuffer::DepthRenderBuffer,
        backface: glium::framebuffer::DepthRenderBuffer,
    }

    let depth_buffers = DepthBuffers {
        backface: glium::framebuffer::DepthRenderBuffer::new(
            &display,
            glium::texture::DepthFormat::F32,
            1024,
            1024,
        )
        .unwrap(),
        frontface: glium::framebuffer::DepthRenderBuffer::new(
            &display,
            glium::texture::DepthFormat::F32,
            1024,
            1024,
        )
        .unwrap(),
    };

    let (volume_tex, names) = {
        let files =
            std::fs::read_dir("data").expect("Folder named data not found in this directory");

        let mut volume_tex = Vec::new();
        let mut names = Vec::new();

        for file in files {
            let file = file.unwrap();

            names.push(imgui::ImString::new(
                file.file_name().into_string().unwrap(),
            ));

            let path = file.path();

            let data = vtk_parser::read_file(path).unwrap();
            let data = data.structured_points().unwrap();

            let image = glium::texture::RawImage3d {
                data: std::borrow::Cow::Owned(data.data.clone()),
                width: data.dims.0,
                height: data.dims.1,
                depth: data.dims.2,
                format: glium::texture::ClientFormat::U8,
            };

            volume_tex.push(
                Texture3d::with_mipmaps(&display, image, glium::texture::MipmapsOption::NoMipmap)
                    .unwrap(),
            );
        }
        (volume_tex, names)
    };

    let mut state = support::State::default();

    let (width, height) = display.get_framebuffer_dimensions();

    let znear = 0.1;
    let zfar = 10.0;
    let mut camera = support::Camera::new(width, height, znear, zfar);

    let mut imgui = imgui::Context::create();
    let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);

    {
        use imgui_winit_support::HiDpiMode;
        let gl_window = display.gl_window();
        let window = gl_window.window();
        platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Rounded);
    }
    let mut renderer = Renderer::init(&mut imgui, &display).unwrap();

    let mut last_frame = std::time::Instant::now();

    events_loop.run(move |ev, _, cf| {
        use glium::glutin::event::Event;
        match ev {
            Event::NewEvents(_) => {
                let now = std::time::Instant::now();
                imgui.io_mut().update_delta_time(now - last_frame);
                last_frame = now;
            }
            Event::MainEventsCleared => {
                let gl_window = display.gl_window();
                platform.prepare_frame(imgui.io_mut(), &gl_window.window()).unwrap();
                gl_window.window().request_redraw();

            }
            Event::RedrawRequested(_) => {
                let mut backface_buffer = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(
                    &display,
                    &textures.backface,
                    &depth_buffers.backface,
                )
                .unwrap();
                let mut frontface_buffer = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(
                    &display,
                    &textures.frontface,
                    &depth_buffers.frontface,
                )
                .unwrap();

                let view = camera.view_matrix();

                let projection: Matrix4<f32> = if state.perspective_selection == 1 {
                    camera.orthographic.into()
                } else {
                    camera.perspective.into()
                };
                let vp: [[f32; 4]; 4] = (projection * view).into();

                backface_buffer.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 0.0);

                let params = glium::DrawParameters {
                    backface_culling: glium::draw_parameters::BackfaceCullingMode::CullCounterClockwise,
                    depth: glium::draw_parameters::Depth {
                        test: glium::draw_parameters::DepthTest::IfMore,
                        write: true,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                backface_buffer
                    .draw(
                        (&cube_pos, cube_models.per_instance().unwrap()),
                        &cube_ind,
                        &cube_prog,
                        &uniform! { u_mvp : vp },
                        &params,
                    )
                    .unwrap();

                frontface_buffer.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);

                let params = glium::DrawParameters {
                    backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
                    depth: glium::draw_parameters::Depth {
                        test: glium::draw_parameters::DepthTest::IfLess,
                        write: true,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                frontface_buffer
                    .draw(
                        (&cube_pos, cube_models.per_instance().unwrap()),
                        &cube_ind,
                        &cube_prog,
                        &uniform! { u_mvp : vp },
                        &params,
                    )
                    .unwrap();

                let mut target = display.draw();
                target.clear_color_and_depth((state.background[0], state.background[1], state.background[2], 0.0), 1.0);

                let params = glium::DrawParameters {
                    blend: glium::Blend {
                        color: glium::BlendingFunction::Max,
                        ..Default::default()
                    },
                    ..Default::default()
                };

                let uniforms = uniform! {
                    u_back : &textures.backface,
                    u_front: &textures.frontface,
                    u_volume: volume_tex[state.selection].sampled().wrap_function(glium::uniforms::SamplerWrapFunction::Clamp).magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear),
                    u_noise: &textures.noise,
                    u_use_noise: state.noise,
                    u_gamma: state.gamma,

                    u_steps: state.steps,
                    u_colour: state.mip_colour,
                    u_dx: state.dx,
                    u_mode: state.mip_or_iso,

                    u_iso: state.isovalue,
                    u_dr: state.grad_step,

                    u_ambient: state.amb_colour,
                    u_amb_str: state.amb_str,
                    u_diffuse: state.dif_colour,
                    u_dif_str: state.dif_str,
                    u_specular: state.spe_colour,
                    u_spe_str: state.spe_str,
                    u_alpha: state.alpha,
                    u_L: [state.light[0].sin()*state.light[1].cos(), state.light[0].sin()*state.light[0].sin(), state.light[0].cos()]
                };

                target
                    .draw(&quad_pos, &quad_ind, &quad_prog, &uniforms, &params)
                    .unwrap();

                // Dear ImGui related
                let frame_rate = imgui.io().framerate;
                state.frame_rate = frame_rate;
                let ui = imgui.frame();
                let gl_window = display.gl_window();

                support::gui(&ui, &mut state, &mut camera, &names);

                platform.prepare_render(&ui, gl_window.window());
                let draw_data = ui.render();
                renderer.render(&mut target, draw_data).unwrap();

                target.finish().unwrap();
            }
            Event::WindowEvent { event: glium::glutin::event::WindowEvent::CloseRequested, .. } => {
                *cf = glium::glutin::event_loop::ControlFlow::Exit;
            }
            event => {
                let gl_window = display.gl_window();
                platform.handle_event(imgui.io_mut(), gl_window.window(), &event);
                camera.handle(event);
            }
        }
    })
}
