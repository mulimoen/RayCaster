use glium::glutin::event::ElementState::Pressed;
use glium::glutin::event::Event;
use glium::glutin::event::MouseButton;

pub struct Camera {
    pressed: (bool, bool, bool),
    mouse_pressed: [bool; 2],
    prev_mouse: (i32, i32),
    pub camera_lock: bool,
    mouse_pos: (i32, i32),
    pub orthographic: cgmath::Ortho<f32>,
    pub perspective: cgmath::PerspectiveFov<f32>,
    pub arcball_camera: arcball::ArcballCamera<f32>,
}

impl Camera {
    pub fn new(w: u32, h: u32, znear: f32, zfar: f32) -> Self {
        Self {
            pressed: (false, false, false),
            mouse_pressed: [false, false],
            prev_mouse: (0, 0),
            camera_lock: false,
            mouse_pos: (0, 0),
            orthographic: {
                let (ww, hh) = if w > h {
                    (2.0 * w as f32 / h as f32, 2.0)
                } else {
                    (2.0, 2.0 * h as f32 / w as f32)
                };
                cgmath::Ortho {
                    left: -ww,
                    right: ww,
                    bottom: -hh,
                    top: hh,
                    near: znear,
                    far: zfar,
                }
            },
            perspective: cgmath::PerspectiveFov {
                fovy: cgmath::Deg(70.0).into(),
                aspect: w as f32 / h as f32,
                near: znear,
                far: zfar,
            },
            arcball_camera: arcball::ArcballCamera::new(
                cgmath::Vector3::new(0.0, 0.0, 0.0),
                4.0,
                [w as f32, h as f32],
            ),
        }
    }
    pub fn handle(&mut self, ev: Event<()>) {
        use glium::glutin::event::WindowEvent::*;
        if let glium::glutin::event::Event::WindowEvent { event, .. } = ev {
            match event {
                MouseInput { state, button, .. } => match button {
                    MouseButton::Left => {
                        self.pressed.0 = state == Pressed;
                        self.mouse_pressed[0] = self.pressed.0;
                    }
                    MouseButton::Right => {
                        self.pressed.1 = state == Pressed;
                        self.mouse_pressed[1] = self.pressed.1;
                    }
                    MouseButton::Middle => {
                        self.pressed.2 = state == Pressed;
                    }
                    _ => {}
                },
                CursorMoved {
                    position: glium::glutin::dpi::PhysicalPosition { x, y },
                    ..
                } => {
                    self.mouse_pos = (x as i32, y as i32);
                    let prev = self.prev_mouse;
                    self.prev_mouse = self.mouse_pos;

                    if self.mouse_pressed[0] & !self.camera_lock {
                        self.arcball_camera.rotate(
                            cgmath::Vector2::new(prev.0 as f32, prev.1 as f32),
                            cgmath::Vector2::new(x as f32, y as f32),
                        );
                    } else if self.mouse_pressed[1] & !self.camera_lock {
                        let mouse_delta = cgmath::Vector2::new(
                            x as f32 - prev.0 as f32,
                            -(y as f32 - prev.1 as f32),
                        );
                        self.arcball_camera.pan(mouse_delta, 0.16);
                    }
                }
                MouseWheel { delta, .. } => {
                    use glium::glutin::event::MouseScrollDelta::*;
                    match delta {
                        LineDelta(_, y) => {
                            if !self.camera_lock {
                                self.arcball_camera.zoom(y, 0.16);
                            }
                        }
                        PixelDelta(glium::glutin::dpi::LogicalPosition { x: _, y }) => {
                            if !self.camera_lock {
                                self.arcball_camera.zoom(y as f32, 0.16);
                            }
                        }
                    };
                }
                Resized(glium::glutin::dpi::PhysicalSize {
                    width: w,
                    height: h,
                }) => {
                    self.perspective.aspect = w as f32 / h as f32;

                    if w > h {
                        self.orthographic.left = -2.0 * w as f32 / h as f32;
                        self.orthographic.right = 2.0 * w as f32 / h as f32;
                        self.orthographic.bottom = -2.0;
                        self.orthographic.top = 2.0;
                    } else {
                        self.orthographic.left = -2.0;
                        self.orthographic.right = 2.0;
                        self.orthographic.bottom = -2.0 * h as f32 / w as f32;
                        self.orthographic.top = 2.0 * h as f32 / w as f32;
                    }
                    self.arcball_camera.update_screen(w as f32, h as f32);
                }
                _ => {}
            }
        }
    }

    pub fn view_matrix(&self) -> cgmath::Matrix4<f32> {
        self.arcball_camera.get_mat4()
    }
}
pub struct State {
    pub steps: i32,
    pub dx: f32,
    pub background: [f32; 3],
    pub selection: usize,
    pub noise: bool,
    pub gamma: f32,
    pub mip_or_iso: i32,
    pub mip_colour: [f32; 3],
    pub isovalue: f32,
    pub amb_colour: [f32; 3],
    pub amb_str: f32,
    pub dif_colour: [f32; 3],
    pub dif_str: f32,
    pub spe_colour: [f32; 3],
    pub spe_str: f32,
    pub alpha: f32,
    pub light: [f32; 2],
    pub grad_step: f32,
    pub perspective_selection: usize,
    pub frame_rate: f32,
}

pub fn gui(ui: &imgui::Ui, state: &mut State, camera: &mut Camera, names: &[imgui::ImString]) {
    use imgui::im_str;
    imgui::Window::new(im_str!("Graphics options"))
        .resizable(true)
        .collapsible(true)
        .movable(true)
        .size([300.0, 100.0], imgui::Condition::FirstUseEver)
        .build(&ui, || {
            imgui::Slider::new(im_str!("Maximum number of steps"))
                .range(0..=400)
                .build(&ui, &mut state.steps);
            imgui::Slider::new(im_str!("Step size"))
                .range(0.0..=0.05)
                .build(&ui, &mut state.dx);
            imgui::Slider::new(im_str!("Gamma factor"))
                .range(0.4..=3.0)
                .build(&ui, &mut state.gamma);
            imgui::ColorEdit::new(im_str!("Background colour"), &mut state.background).build(&ui);
            ui.text(im_str!("Projection:"));
            ui.same_line(0.0);
            ui.radio_button(im_str!("Perspective"), &mut state.perspective_selection, 0);
            ui.same_line(0.0);
            ui.radio_button(im_str!("Orthographic"), &mut state.perspective_selection, 1);

            ui.checkbox(im_str!("Lock camera"), &mut camera.camera_lock);
            ui.checkbox(im_str!("Use noise texture"), &mut state.noise);

            if ui.small_button(im_str!("Volume dataset:")) {
                ui.open_popup(im_str!("Select:"));
            }
            ui.same_line(0.0);
            ui.text(&names[state.selection]);
            ui.popup(im_str!("Select:"), || {
                for (index, name) in names.iter().enumerate() {
                    if imgui::Selectable::new(name)
                        .flags(imgui::SelectableFlags::empty())
                        .selected(false)
                        .size([0.0, 0.0])
                        .build(&ui)
                    {
                        state.selection = index;
                    }
                }
            });

            ui.text(im_str!("Framerate: {:.2}", state.frame_rate));

            ui.text(im_str!("Select projection mode:"));
            ui.same_line(0.0);
            ui.radio_button(im_str!("MIP"), &mut state.mip_or_iso, 0);
            ui.same_line(0.0);
            ui.radio_button(im_str!("ISO"), &mut state.mip_or_iso, 1);

            if imgui::CollapsingHeader::new(im_str!("Maximum Intensity Projection")).build(&ui) {
                imgui::ColorEdit::new(im_str!("MIP colour"), &mut state.mip_colour).build(&ui);
            }

            if imgui::CollapsingHeader::new(im_str!("Isosurface Extraction")).build(&ui) {
                imgui::Slider::new(im_str!("Isovalue"))
                    .range(0.0..=1.0)
                    .build(&ui, &mut state.isovalue);
                imgui::Slider::new(im_str!("Gradient step length"))
                    .range(0.0..=1.0 / 10.0)
                    .build(&ui, &mut state.grad_step);

                ui.separator();

                imgui::ColorEdit::new(im_str!("Ambient colour"), &mut state.amb_colour).build(&ui);
                imgui::Slider::new(im_str!("Ambient strength"))
                    .range(0.0..=1.0)
                    .build(&ui, &mut state.amb_str);

                imgui::ColorEdit::new(im_str!("Diffuse colour"), &mut state.dif_colour).build(&ui);
                imgui::Slider::new(im_str!("Diffuse strength"))
                    .range(0.0..=1.0)
                    .build(&ui, &mut state.dif_str);

                imgui::ColorEdit::new(im_str!("Specular colour"), &mut state.spe_colour).build(&ui);
                imgui::Slider::new(im_str!("Specular strength"))
                    .range(0.0..=0.03)
                    .build(&ui, &mut state.spe_str);
                imgui::Slider::new(im_str!("Specular alpha"))
                    .range(10.0..=900.0)
                    .build(&ui, &mut state.alpha);

                ui.separator();

                imgui::Slider::new(im_str!("Light vector theta"))
                    .range(0.0..=std::f32::consts::PI)
                    .build(&ui, &mut state.light[0]);
                imgui::Slider::new(im_str!("Light vector phi"))
                    .range(0.0..=2.0 * std::f32::consts::PI)
                    .build(&ui, &mut state.light[1]);
            }
        });
}
