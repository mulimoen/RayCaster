use arcball;
use cgmath;
use glium;
// use imgui;

use glium::glutin::event::ElementState::Pressed;
use glium::glutin::event::Event;
use glium::glutin::event::MouseButton;

pub struct Support {
    pressed: (bool, bool, bool),
    mouse_pressed: [bool; 2],
    prev_mouse: (i32, i32),
    pub camera_lock: bool,
    keys_pressed: Vec<char>,
    mouse_pos: (i32, i32),
    mouse_wheel: f32,
    pub orthographic: cgmath::Ortho<f32>,
    pub perspective: cgmath::PerspectiveFov<f32>,
    pub arcball_camera: arcball::ArcballCamera<f32>,
}

impl Support {
    pub fn new(w: u32, h: u32, znear: f32, zfar: f32) -> Support {
        Support {
            pressed: (false, false, false),
            mouse_pressed: [false, false],
            prev_mouse: (0, 0),
            camera_lock: false,
            keys_pressed: Vec::new(),
            mouse_pos: (0, 0),
            mouse_wheel: 0.0,
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
    pub fn handle(
        &mut self,
        ev: Event<()>,
        cf: &mut glium::glutin::event_loop::ControlFlow,
    ) -> bool {
        use glium::glutin::event::WindowEvent::*;
        if let glium::glutin::event::Event::WindowEvent { event, .. } = ev {
            match event {
                CloseRequested | Destroyed => {
                    *cf = glium::glutin::event_loop::ControlFlow::Exit;
                    return true;
                }
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
                KeyboardInput { input: x, .. } => {
                    self.keys_pressed.push(x.scancode as u8 as char);
                }
                MouseWheel { delta, .. } => {
                    use glium::glutin::event::MouseScrollDelta::*;
                    match delta {
                        LineDelta(_, y) => {
                            self.mouse_wheel = y;
                            if !self.camera_lock {
                                self.arcball_camera.zoom(y, 0.16);
                            }
                        }
                        PixelDelta(glium::glutin::dpi::LogicalPosition { x: _, y }) => {
                            self.mouse_wheel = y as f32;
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
        return false;
    }

    fn clear(&mut self) {
        self.keys_pressed.clear();
        self.mouse_wheel = 0.0;
    }

    pub fn pass_to_imgui(&mut self, imgui: &mut imgui::Io) {
        imgui.mouse_pos = [self.mouse_pos.0 as f32, self.mouse_pos.1 as f32];
        imgui.mouse_down = [self.pressed.0, self.pressed.1, self.pressed.2, false, false];
        for e in &self.keys_pressed {
            imgui.add_input_character(*e);
        }
        imgui.mouse_wheel = self.mouse_wheel;
        self.clear();
    }

    pub fn view_matrix(&self) -> cgmath::Matrix4<f32> {
        self.arcball_camera.get_mat4()
    }
}
