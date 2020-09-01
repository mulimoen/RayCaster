use cgmath::Matrix4;
use glium::{
    texture::{Texture2d, Texture3d},
    uniform, IndexBuffer, Program, Surface, VertexBuffer,
};
use imgui::im_str;
use imgui_glium_renderer::Renderer;

mod cube;
mod raycast;
mod support;

fn main() {
    let events_loop = glium::glutin::event_loop::EventLoop::new();
    let window = glium::glutin::window::WindowBuilder::new();
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_depth_buffer(24);
    let display = glium::Display::new(window, context, &events_loop).unwrap();

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

    let backface_tex = Texture2d::empty_with_format(
        &display,
        glium::texture::UncompressedFloatFormat::F32F32F32F32,
        glium::texture::MipmapsOption::NoMipmap,
        1024,
        1024,
    )
    .unwrap();

    let depth_tex_back = glium::framebuffer::DepthRenderBuffer::new(
        &display,
        glium::texture::DepthFormat::F32,
        1024,
        1024,
    )
    .unwrap();

    let frontface_tex = Texture2d::empty_with_format(
        &display,
        glium::texture::UncompressedFloatFormat::F32F32F32F32,
        glium::texture::MipmapsOption::NoMipmap,
        1024,
        1024,
    )
    .unwrap();
    let depth_tex_front = glium::framebuffer::DepthRenderBuffer::new(
        &display,
        glium::texture::DepthFormat::F32,
        1024,
        1024,
    )
    .unwrap();

    let noise_tex = {
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

    let mut steps: i32 = 200;
    let mut dx: f32 = 0.01;
    let mut background: [f32; 3] = [0.0, 0.0, 0.0];

    let mut selection: usize = 0;
    let mut noise: bool = true;
    let mut gamma: f32 = 2.2;

    let mut mip_or_iso: i32 = 0;

    let mut mip_colour: [f32; 3] = [1.0, 1.0, 1.0];

    let mut isovalue: f32 = 0.3;
    let mut amb_colour: [f32; 3] = [1.0, 0.0, 0.0];
    let mut amb_str: f32 = 0.1;
    let mut dif_colour: [f32; 3] = [1.0, 0.0, 0.0];
    let mut dif_str: f32 = 1.0;
    let mut spe_colour: [f32; 3] = [1.0, 1.0, 1.0];
    let mut spe_str: f32 = 0.005;
    let mut alpha: f32 = 300.0;

    let mut light: [f32; 2] = [std::f32::consts::PI / 2.0, 0.0];
    let mut grad_step: f32 = 5.0 / 256.0;

    let (width, height) = display.get_framebuffer_dimensions();

    let znear = 0.1;
    let zfar = 10.0;
    let mut input = support::Support::new(width, height, znear, zfar);

    let mut perspective_selection = 0;

    let mut imgui = imgui::Context::create();
    imgui.io_mut().mouse_pos = [0.0, 0.0];

    let mut renderer = Renderer::init(&mut imgui, &display).unwrap();

    let mut last_frame = std::time::Instant::now();

    events_loop.run(move |ev, _, cf| {
        if input.handle(ev, cf) {
            return
        }
        let now = std::time::Instant::now();
        let dt = now - last_frame;
        if dt < std::time::Duration::from_millis(50) {
            *cf = glium::glutin::event_loop::ControlFlow::Poll;
            return;
        }
        *cf = glium::glutin::event_loop::ControlFlow::Wait;
        last_frame = now;

        let mut backface_buffer = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(
            &display,
            &backface_tex,
            &depth_tex_back,
        )
        .unwrap();
        let mut frontface_buffer = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(
            &display,
            &frontface_tex,
            &depth_tex_front,
        )
        .unwrap();

        input.pass_to_imgui(imgui.io_mut());

        let view = input.view_matrix();

        let projection: Matrix4<f32> = if perspective_selection == 1 {
            input.orthographic.into()
        } else {
            input.perspective.into()
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
        target.clear_color_and_depth((background[0], background[1], background[2], 0.0), 1.0);

        let params = glium::DrawParameters {
            blend: glium::Blend {
                color: glium::BlendingFunction::Max,
                ..Default::default()
            },
            ..Default::default()
        };

        let uniforms = uniform! {
            u_back : &backface_tex,
            u_front: &frontface_tex,
            u_volume: volume_tex[selection].sampled().wrap_function(glium::uniforms::SamplerWrapFunction::Clamp).magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear),
            u_noise: &noise_tex,
            u_use_noise: noise,
            u_gamma: gamma,

            u_steps: steps,
            u_colour: mip_colour,
            u_dx: dx,
            u_mode: mip_or_iso,

            u_iso: isovalue,
            u_dr: grad_step,

            u_ambient: amb_colour,
            u_amb_str: amb_str,
            u_diffuse: dif_colour,
            u_dif_str: dif_str,
            u_specular: spe_colour,
            u_spe_str: spe_str,
            u_alpha: alpha,
            u_L: [light[0].sin()*light[1].cos(), light[0].sin()*light[0].sin(), light[0].cos()]
        };

        target
            .draw(&quad_pos, &quad_ind, &quad_prog, &uniforms, &params)
            .unwrap();

        // The following part is all related to Dear ImGui
        let w = display.gl_window();
        let window = w.window();
        let size_pixels = window.inner_size();


        let frame_rate = imgui.io().framerate;
        imgui.io_mut().display_size = [size_pixels.width as _, size_pixels.height as _];
        imgui.io_mut().delta_time = dt.as_secs() as f32 + dt.subsec_nanos() as f32 * 1e-9;
        let ui = imgui.frame();

        imgui::Window::new(im_str!("Graphics options"))
            .resizable(true)
            .collapsible(true)
            .movable(true)
            .size([300.0, 100.0], imgui::Condition::FirstUseEver)
            .build(&ui, || {
                imgui::Slider::new(im_str!("Maximum number of steps"), 0..=400)
                    .build(&ui, &mut steps);
                imgui::Slider::new(im_str!("Step size"), 0.0..=0.05)
                    .build(&ui, &mut dx);
                imgui::Slider::new(im_str!("Gamma factor"), 0.4..=3.0)
                    .build(&ui, &mut gamma);
                imgui::ColorEdit::new(im_str!("Background colour"), &mut background).build(&ui);
                ui.text(im_str!("Projection:"));
                ui.same_line(0.0);
                ui.radio_button(im_str!("Perspective"), &mut perspective_selection, 0);
                ui.same_line(0.0);
                ui.radio_button(im_str!("Orthographic"), &mut perspective_selection, 1);

                ui.checkbox(im_str!("Lock camera"), &mut input.camera_lock);
                ui.checkbox(im_str!("Use noise texture"), &mut noise);

                if ui.small_button(im_str!("Volume dataset:")) {
                    ui.open_popup(im_str!("Select:"));
                }
                ui.same_line(0.0);
                ui.text(&names[selection]);
                ui.popup(im_str!("Select:"), || {
                    for (index, name) in names.iter().enumerate() {
                        if imgui::Selectable::new(name).flags(imgui::SelectableFlags::empty()).selected(false).size([0.0, 0.0]).build(&ui) {
                            selection = index;
                        }
                    }
                });

                ui.text(im_str!("Framerate: {:.2}", frame_rate));

                ui.text(im_str!("Select projection mode:"));
                ui.same_line(0.0);
                ui.radio_button(im_str!("MIP"), &mut mip_or_iso, 0);
                ui.same_line(0.0);
                ui.radio_button(im_str!("ISO"), &mut mip_or_iso, 1);

                if imgui::CollapsingHeader::new(im_str!("Maximum Intensity Projection")).build(&ui) {
                    imgui::ColorEdit::new(im_str!("MIP colour"), &mut mip_colour).build(&ui);
                }

                if imgui::CollapsingHeader::new(im_str!("Isosurface Extraction")).build(&ui) {
                    imgui::Slider::new(im_str!("Isovalue"), 0.0..=1.0).build(&ui, &mut isovalue);
                    imgui::Slider::new(im_str!("Gradient step length"), 0.0..=1.0/10.0).build(&ui, &mut grad_step);

                    ui.separator();

                    imgui::ColorEdit::new(im_str!("Ambient colour"), &mut amb_colour).build(&ui);
                    imgui::Slider::new(im_str!("Ambient strength"), 0.0..=1.0).build(&ui, &mut amb_str);

                    imgui::ColorEdit::new(im_str!("Diffuse colour"), &mut dif_colour).build(&ui);
                    imgui::Slider::new(im_str!("Diffuse strength"), 0.0..=1.0).build(&ui, &mut dif_str);

                    imgui::ColorEdit::new(im_str!("Specular colour"), &mut spe_colour).build(&ui);
                    imgui::Slider::new(im_str!("Specular strength"),0.0..=0.03).build(&ui, &mut spe_str);
                    imgui::Slider::new(im_str!("Specular alpha"), 10.0..=900.0).build(&ui, &mut alpha);

                    ui.separator();

                    imgui::Slider::new(im_str!("Light vector theta"), 0.0..=std::f32::consts::PI).build(&ui, &mut light[0]);
                    imgui::Slider::new(im_str!("Light vector phi"), 0.0..=2.0 * std::f32::consts::PI).build(&ui, &mut light[1]);
                }
            });

        let draw_data = ui.render();
        renderer.render(&mut target, draw_data).unwrap();

        target.finish().unwrap();
    })
}
