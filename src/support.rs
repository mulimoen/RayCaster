use glium::glutin::event::ElementState::Pressed;
use glium::glutin::event::Event;
use glium::glutin::event::MouseButton;

pub struct Support {
    pressed: (bool, bool, bool),
    mouse_pressed: [bool; 2],
    prev_mouse: (i32, i32),
    pub camera_lock: bool,
    mouse_pos: (i32, i32),
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
