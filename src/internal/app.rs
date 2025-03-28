use std::sync::{Arc, Once};

use crate::{executor, graphics, keyboard, mouse};

pub(crate) struct Config {
    pub title: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self { title: None }
    }
}

static ONCE: Once = Once::new();

pub(crate) fn run(config: Config, executor: executor::Executor) {
    ONCE.call_once(|| {
        let event_loop = winit::event_loop::EventLoop::builder().build().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        let mut app = App::new(config, executor);
        let _ = event_loop.run_app(&mut app);
    });
}

pub(crate) struct App {
    config: Config,
    internals: Option<Internals>,
    executor: executor::Executor,
}

impl App {
    pub(crate) fn new(config: Config, executor: executor::Executor) -> Self {
        Self {
            config,
            internals: None,
            executor,
        }
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(internals) = Internals::new(&mut self.config, event_loop) {
            self.internals = Some(internals);
            self.executor.load();
        } else {
            event_loop.exit();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let Some(internals) = self.internals.as_mut() else {
            return;
        };
        match event {
            winit::event::WindowEvent::RedrawRequested => {
                graphics::context::reset();
                self.executor.draw();
                if let Ok(surface_texture) = internals.surface.get_current_texture() {
                    let view = surface_texture
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    graphics::context::render(view);
                    surface_texture.present();
                }
            }

            winit::event::WindowEvent::Resized(size) => internals.resize(size.width, size.height),

            winit::event::WindowEvent::CloseRequested => event_loop.exit(),

            winit::event::WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        repeat: false,
                        physical_key,
                        state,
                        ..
                    },
                ..
            } => {
                let scancode = physical_key_to_scancode(physical_key);
                keyboard::state::set_key(scancode, state.is_pressed());
                if state.is_pressed() {
                    self.executor.key_pressed(scancode);
                } else {
                    self.executor.key_released(scancode);
                }
            }

            winit::event::WindowEvent::CursorMoved { position, .. } => {
                let (x_0, y_0) = mouse::state::get_position();
                mouse::state::set_position(position.x as f32, position.y as f32);
                self.executor.mouse_moved(
                    position.x as f32,
                    position.y as f32,
                    position.x as f32 - x_0,
                    position.y as f32 - y_0,
                );
            }

            winit::event::WindowEvent::MouseInput {
                state,
                button:
                    button @ (winit::event::MouseButton::Left
                    | winit::event::MouseButton::Right
                    | winit::event::MouseButton::Middle),
                ..
            } => {
                let button = match button {
                    winit::event::MouseButton::Left => mouse::Button::Left,
                    winit::event::MouseButton::Right => mouse::Button::Right,
                    winit::event::MouseButton::Middle => mouse::Button::Middle,
                    _ => unreachable!(),
                };
                let (x, y) = mouse::state::get_position();
                mouse::state::set_button(button, state.is_pressed());
                if state.is_pressed() {
                    self.executor.mouse_pressed(x, y, button);
                } else {
                    self.executor.mouse_released(x, y, button);
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _: &winit::event_loop::ActiveEventLoop) {
        self.executor.update();
        let Some(internals) = self.internals.as_mut() else {
            return;
        };
        internals.window.request_redraw();
    }
}

struct Internals {
    window: Arc<winit::window::Window>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
}

impl Internals {
    fn new(config: &mut Config, event_loop: &winit::event_loop::ActiveEventLoop) -> Option<Self> {
        let attributes = winit::window::Window::default_attributes()
            .with_title(config.title.take().unwrap_or("heart".into()));
        let window = Arc::new(event_loop.create_window(attributes).ok()?);

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        let surface = instance.create_surface(window.clone()).ok()?;

        let adapter = instance
            .enumerate_adapters(wgpu::Backends::all())
            .into_iter()
            .filter(|adapter| adapter.is_surface_supported(&surface))
            .next()?;

        let width = window.inner_size().width;
        let height = window.inner_size().height;

        if !surface
            .get_capabilities(&adapter)
            .formats
            .contains(&wgpu::TextureFormat::Bgra8UnormSrgb)
        {
            return None;
        }
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: graphics::renderer::TEXTURE_FORMAT,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
        };

        let renderer = graphics::renderer::Renderer::new(adapter)?;
        renderer.set_viewport_uniform(width as f32, height as f32);

        surface.configure(&renderer.device, &surface_config);

        graphics::context::init(renderer);

        Some(Self {
            window,
            surface,
            surface_config,
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            let context = graphics::context::get();
            self.surface
                .configure(&context.renderer.device, &self.surface_config);
            context
                .renderer
                .set_viewport_uniform(width as f32, height as f32);
        }
    }
}

fn physical_key_to_scancode(physical_key: winit::keyboard::PhysicalKey) -> keyboard::Scancode {
    match physical_key {
        winit::keyboard::PhysicalKey::Code(code) => match code {
            winit::keyboard::KeyCode::Backquote => keyboard::Scancode::Backquote,
            winit::keyboard::KeyCode::Backslash => keyboard::Scancode::Backslash,
            winit::keyboard::KeyCode::BracketLeft => keyboard::Scancode::BracketLeft,
            winit::keyboard::KeyCode::BracketRight => keyboard::Scancode::BracketRight,
            winit::keyboard::KeyCode::Comma => keyboard::Scancode::Comma,
            winit::keyboard::KeyCode::Digit0 => keyboard::Scancode::Digit0,
            winit::keyboard::KeyCode::Digit1 => keyboard::Scancode::Digit1,
            winit::keyboard::KeyCode::Digit2 => keyboard::Scancode::Digit2,
            winit::keyboard::KeyCode::Digit3 => keyboard::Scancode::Digit3,
            winit::keyboard::KeyCode::Digit4 => keyboard::Scancode::Digit4,
            winit::keyboard::KeyCode::Digit5 => keyboard::Scancode::Digit5,
            winit::keyboard::KeyCode::Digit6 => keyboard::Scancode::Digit6,
            winit::keyboard::KeyCode::Digit7 => keyboard::Scancode::Digit7,
            winit::keyboard::KeyCode::Digit8 => keyboard::Scancode::Digit8,
            winit::keyboard::KeyCode::Digit9 => keyboard::Scancode::Digit9,
            winit::keyboard::KeyCode::Equal => keyboard::Scancode::Equal,
            winit::keyboard::KeyCode::IntlBackslash => keyboard::Scancode::IntlBackslash,
            winit::keyboard::KeyCode::IntlRo => keyboard::Scancode::IntlRo,
            winit::keyboard::KeyCode::IntlYen => keyboard::Scancode::IntlYen,
            winit::keyboard::KeyCode::KeyA => keyboard::Scancode::KeyA,
            winit::keyboard::KeyCode::KeyB => keyboard::Scancode::KeyB,
            winit::keyboard::KeyCode::KeyC => keyboard::Scancode::KeyC,
            winit::keyboard::KeyCode::KeyD => keyboard::Scancode::KeyD,
            winit::keyboard::KeyCode::KeyE => keyboard::Scancode::KeyE,
            winit::keyboard::KeyCode::KeyF => keyboard::Scancode::KeyF,
            winit::keyboard::KeyCode::KeyG => keyboard::Scancode::KeyG,
            winit::keyboard::KeyCode::KeyH => keyboard::Scancode::KeyH,
            winit::keyboard::KeyCode::KeyI => keyboard::Scancode::KeyI,
            winit::keyboard::KeyCode::KeyJ => keyboard::Scancode::KeyJ,
            winit::keyboard::KeyCode::KeyK => keyboard::Scancode::KeyK,
            winit::keyboard::KeyCode::KeyL => keyboard::Scancode::KeyL,
            winit::keyboard::KeyCode::KeyM => keyboard::Scancode::KeyM,
            winit::keyboard::KeyCode::KeyN => keyboard::Scancode::KeyN,
            winit::keyboard::KeyCode::KeyO => keyboard::Scancode::KeyO,
            winit::keyboard::KeyCode::KeyP => keyboard::Scancode::KeyP,
            winit::keyboard::KeyCode::KeyQ => keyboard::Scancode::KeyQ,
            winit::keyboard::KeyCode::KeyR => keyboard::Scancode::KeyR,
            winit::keyboard::KeyCode::KeyS => keyboard::Scancode::KeyS,
            winit::keyboard::KeyCode::KeyT => keyboard::Scancode::KeyT,
            winit::keyboard::KeyCode::KeyU => keyboard::Scancode::KeyU,
            winit::keyboard::KeyCode::KeyV => keyboard::Scancode::KeyV,
            winit::keyboard::KeyCode::KeyW => keyboard::Scancode::KeyW,
            winit::keyboard::KeyCode::KeyX => keyboard::Scancode::KeyX,
            winit::keyboard::KeyCode::KeyY => keyboard::Scancode::KeyY,
            winit::keyboard::KeyCode::KeyZ => keyboard::Scancode::KeyZ,
            winit::keyboard::KeyCode::Minus => keyboard::Scancode::Minus,
            winit::keyboard::KeyCode::Period => keyboard::Scancode::Period,
            winit::keyboard::KeyCode::Quote => keyboard::Scancode::Quote,
            winit::keyboard::KeyCode::Semicolon => keyboard::Scancode::Semicolon,
            winit::keyboard::KeyCode::Slash => keyboard::Scancode::Slash,
            winit::keyboard::KeyCode::AltLeft => keyboard::Scancode::AltLeft,
            winit::keyboard::KeyCode::AltRight => keyboard::Scancode::AltRight,
            winit::keyboard::KeyCode::Backspace => keyboard::Scancode::Backspace,
            winit::keyboard::KeyCode::CapsLock => keyboard::Scancode::CapsLock,
            winit::keyboard::KeyCode::ContextMenu => keyboard::Scancode::ContextMenu,
            winit::keyboard::KeyCode::ControlLeft => keyboard::Scancode::ControlLeft,
            winit::keyboard::KeyCode::ControlRight => keyboard::Scancode::ControlRight,
            winit::keyboard::KeyCode::Enter => keyboard::Scancode::Enter,
            winit::keyboard::KeyCode::SuperLeft => keyboard::Scancode::MetaLeft,
            winit::keyboard::KeyCode::SuperRight => keyboard::Scancode::MetaRight,
            winit::keyboard::KeyCode::ShiftLeft => keyboard::Scancode::ShiftLeft,
            winit::keyboard::KeyCode::ShiftRight => keyboard::Scancode::ShiftRight,
            winit::keyboard::KeyCode::Space => keyboard::Scancode::Space,
            winit::keyboard::KeyCode::Tab => keyboard::Scancode::Tab,
            winit::keyboard::KeyCode::Convert => keyboard::Scancode::Convert,
            winit::keyboard::KeyCode::KanaMode => keyboard::Scancode::KanaMode,
            winit::keyboard::KeyCode::NonConvert => keyboard::Scancode::NonConvert,
            winit::keyboard::KeyCode::Delete => keyboard::Scancode::Delete,
            winit::keyboard::KeyCode::End => keyboard::Scancode::End,
            winit::keyboard::KeyCode::Help => keyboard::Scancode::Help,
            winit::keyboard::KeyCode::Home => keyboard::Scancode::Home,
            winit::keyboard::KeyCode::Insert => keyboard::Scancode::Insert,
            winit::keyboard::KeyCode::PageDown => keyboard::Scancode::PageDown,
            winit::keyboard::KeyCode::PageUp => keyboard::Scancode::PageUp,
            winit::keyboard::KeyCode::ArrowDown => keyboard::Scancode::ArrowDown,
            winit::keyboard::KeyCode::ArrowLeft => keyboard::Scancode::ArrowLeft,
            winit::keyboard::KeyCode::ArrowRight => keyboard::Scancode::ArrowRight,
            winit::keyboard::KeyCode::ArrowUp => keyboard::Scancode::ArrowUp,
            winit::keyboard::KeyCode::NumLock => keyboard::Scancode::NumLock,
            winit::keyboard::KeyCode::Numpad0 => keyboard::Scancode::Numpad0,
            winit::keyboard::KeyCode::Numpad1 => keyboard::Scancode::Numpad1,
            winit::keyboard::KeyCode::Numpad2 => keyboard::Scancode::Numpad2,
            winit::keyboard::KeyCode::Numpad3 => keyboard::Scancode::Numpad3,
            winit::keyboard::KeyCode::Numpad4 => keyboard::Scancode::Numpad4,
            winit::keyboard::KeyCode::Numpad5 => keyboard::Scancode::Numpad5,
            winit::keyboard::KeyCode::Numpad6 => keyboard::Scancode::Numpad6,
            winit::keyboard::KeyCode::Numpad7 => keyboard::Scancode::Numpad7,
            winit::keyboard::KeyCode::Numpad8 => keyboard::Scancode::Numpad8,
            winit::keyboard::KeyCode::Numpad9 => keyboard::Scancode::Numpad9,
            winit::keyboard::KeyCode::NumpadAdd => keyboard::Scancode::NumpadAdd,
            winit::keyboard::KeyCode::NumpadDecimal => keyboard::Scancode::NumpadDecimal,
            winit::keyboard::KeyCode::NumpadDivide => keyboard::Scancode::NumpadDivide,
            winit::keyboard::KeyCode::NumpadEnter => keyboard::Scancode::NumpadEnter,
            winit::keyboard::KeyCode::NumpadMultiply => keyboard::Scancode::NumpadMultiply,
            winit::keyboard::KeyCode::NumpadSubtract => keyboard::Scancode::NumpadSubtract,
            winit::keyboard::KeyCode::Escape => keyboard::Scancode::Escape,
            winit::keyboard::KeyCode::PrintScreen => keyboard::Scancode::PrintScreen,
            winit::keyboard::KeyCode::ScrollLock => keyboard::Scancode::ScrollLock,
            winit::keyboard::KeyCode::Pause => keyboard::Scancode::Pause,
            winit::keyboard::KeyCode::F1 => keyboard::Scancode::F1,
            winit::keyboard::KeyCode::F2 => keyboard::Scancode::F2,
            winit::keyboard::KeyCode::F3 => keyboard::Scancode::F3,
            winit::keyboard::KeyCode::F4 => keyboard::Scancode::F4,
            winit::keyboard::KeyCode::F5 => keyboard::Scancode::F5,
            winit::keyboard::KeyCode::F6 => keyboard::Scancode::F6,
            winit::keyboard::KeyCode::F7 => keyboard::Scancode::F7,
            winit::keyboard::KeyCode::F8 => keyboard::Scancode::F8,
            winit::keyboard::KeyCode::F9 => keyboard::Scancode::F9,
            winit::keyboard::KeyCode::F10 => keyboard::Scancode::F10,
            winit::keyboard::KeyCode::F11 => keyboard::Scancode::F11,
            winit::keyboard::KeyCode::F12 => keyboard::Scancode::F12,
            _ => keyboard::Scancode::Unidentified,
        },
        _ => keyboard::Scancode::Unidentified,
    }
}
