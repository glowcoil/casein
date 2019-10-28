use gouache::{Color, Frame, Font, Cache, renderers::GlRenderer};
use casein::{*, input::*};

const FRAME: std::time::Duration = std::time::Duration::from_micros(1_000_000 / 60);

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new()
        .with_dimensions(glutin::dpi::LogicalSize::new(800.0, 600.0))
        .with_title("gouache");
    let context = glutin::ContextBuilder::new()
        .build_windowed(window_builder, &events_loop)
        .unwrap();
    let context = unsafe { context.make_current() }.unwrap();

    gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

    let mut cache = Cache::new();
    let mut renderer = GlRenderer::new();

    let mut font = Font::from_bytes(include_bytes!("../res/SourceSansPro-Regular.ttf")).unwrap();

    let mut root = Elem::new(Box::new(Button::new(
        Box::new(Padding::new(
            5.0,
            Box::new(Text::new(font, 14.0, "jackdaws love my".to_string())),
        )),
    )));

    let mut input_state = InputState {
        mouse_x: 0.0,
        mouse_y: 0.0,
        modifiers: Modifiers::default(),
    };

    let mut running = true;
    let mut now = std::time::Instant::now();
    while running {
        let size = context.window().get_inner_size().unwrap();

        let mut frame = Frame::new(&mut cache, &mut renderer, size.width as f32, size.height as f32);

        frame.clear(Color::rgba(0.1, 0.15, 0.2, 1.0));

        root.layout(size.width as f32, size.height as f32);
        root.offset(0.0, 0.0);
        root.render(&mut frame);

        frame.finish();

        context.swap_buffers().unwrap();

        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => {
                use glutin::WindowEvent::*;
                match event {
                    CloseRequested => running = false,
                    Resized(logical_size) => {
                        let dpi_factor = context.window().get_hidpi_factor();
                        context.resize(logical_size.to_physical(dpi_factor));
                    }
                    CursorMoved { position, .. } => {
                        input_state.mouse_x = position.x as f32;
                        input_state.mouse_y = position.y as f32;
                        root.handle(Input::MouseMove, &input_state);
                    }
                    MouseInput { state, button, modifiers, .. } => {
                        input_state.modifiers = glutin_modifiers(modifiers);
                        let button = match button {
                            glutin::MouseButton::Left => Some(MouseButton:: Left),
                            glutin::MouseButton::Middle => Some(MouseButton::Middle),
                            glutin::MouseButton::Right => Some(MouseButton:: Right),
                            _ => None,
                        };
                        if let Some(button) = button {
                            let input = match state {
                                glutin::ElementState::Pressed => Input::MouseDown(button),
                                glutin::ElementState::Released => Input::MouseUp(button),
                            };
                            root.handle(input, &input_state);
                        }
                    }
                    MouseWheel { delta, modifiers, .. } => {
                        input_state.modifiers = glutin_modifiers(modifiers);
                        let (x, y) = match delta {
                            glutin::MouseScrollDelta::LineDelta(x, y) => (x * 48.0, y * 48.0),
                            glutin::MouseScrollDelta::PixelDelta(glutin::dpi::LogicalPosition { x, y }) => (x as f32, y as f32),
                        };
                        root.handle(Input::Scroll(x, y), &input_state);
                    }
                    KeyboardInput { input, .. } =>  {
                        input_state.modifiers = glutin_modifiers(input.modifiers);
                        if let Some(key) = input.virtual_keycode.and_then(glutin_key) {
                            let input = match input.state {
                                glutin::ElementState::Pressed => Input::KeyDown(key),
                                glutin::ElementState::Released => Input::KeyUp(key),
                            };
                            root.handle(input, &input_state);
                        }
                    }
                    ReceivedCharacter(char) => {
                        root.handle(Input::Char(char), &input_state);
                    }
                    _ => {}
                }
            }
            _ => {}
        });

        let elapsed = now.elapsed();
        if elapsed < FRAME {
            std::thread::sleep(FRAME - elapsed);
        }
        now = std::time::Instant::now();
    }
}

fn glutin_modifiers(modifiers: glutin::ModifiersState) -> Modifiers {
    Modifiers {
        shift: modifiers.shift,
        ctrl: modifiers.ctrl,
        alt: modifiers.alt,
        meta: modifiers.logo,
    }
}

fn glutin_key(key: glutin::VirtualKeyCode) -> Option<Key> {
    match key {
        glutin::VirtualKeyCode::Key1 => Some(Key::Key1),
        glutin::VirtualKeyCode::Key2 => Some(Key::Key2),
        glutin::VirtualKeyCode::Key3 => Some(Key::Key3),
        glutin::VirtualKeyCode::Key4 => Some(Key::Key4),
        glutin::VirtualKeyCode::Key5 => Some(Key::Key5),
        glutin::VirtualKeyCode::Key6 => Some(Key::Key6),
        glutin::VirtualKeyCode::Key7 => Some(Key::Key7),
        glutin::VirtualKeyCode::Key8 => Some(Key::Key8),
        glutin::VirtualKeyCode::Key9 => Some(Key::Key9),
        glutin::VirtualKeyCode::Key0 => Some(Key::Key0),
        glutin::VirtualKeyCode::A => Some(Key::A),
        glutin::VirtualKeyCode::B => Some(Key::B),
        glutin::VirtualKeyCode::C => Some(Key::C),
        glutin::VirtualKeyCode::D => Some(Key::D),
        glutin::VirtualKeyCode::E => Some(Key::E),
        glutin::VirtualKeyCode::F => Some(Key::F),
        glutin::VirtualKeyCode::G => Some(Key::G),
        glutin::VirtualKeyCode::H => Some(Key::H),
        glutin::VirtualKeyCode::I => Some(Key::I),
        glutin::VirtualKeyCode::J => Some(Key::J),
        glutin::VirtualKeyCode::K => Some(Key::K),
        glutin::VirtualKeyCode::L => Some(Key::L),
        glutin::VirtualKeyCode::M => Some(Key::M),
        glutin::VirtualKeyCode::N => Some(Key::N),
        glutin::VirtualKeyCode::O => Some(Key::O),
        glutin::VirtualKeyCode::P => Some(Key::P),
        glutin::VirtualKeyCode::Q => Some(Key::Q),
        glutin::VirtualKeyCode::R => Some(Key::R),
        glutin::VirtualKeyCode::S => Some(Key::S),
        glutin::VirtualKeyCode::T => Some(Key::T),
        glutin::VirtualKeyCode::U => Some(Key::U),
        glutin::VirtualKeyCode::V => Some(Key::V),
        glutin::VirtualKeyCode::W => Some(Key::W),
        glutin::VirtualKeyCode::X => Some(Key::X),
        glutin::VirtualKeyCode::Y => Some(Key::Y),
        glutin::VirtualKeyCode::Z => Some(Key::Z),
        glutin::VirtualKeyCode::Escape => Some(Key::Escape),
        glutin::VirtualKeyCode::F1 => Some(Key::F1),
        glutin::VirtualKeyCode::F2 => Some(Key::F2),
        glutin::VirtualKeyCode::F3 => Some(Key::F3),
        glutin::VirtualKeyCode::F4 => Some(Key::F4),
        glutin::VirtualKeyCode::F5 => Some(Key::F5),
        glutin::VirtualKeyCode::F6 => Some(Key::F6),
        glutin::VirtualKeyCode::F7 => Some(Key::F7),
        glutin::VirtualKeyCode::F8 => Some(Key::F8),
        glutin::VirtualKeyCode::F9 => Some(Key::F9),
        glutin::VirtualKeyCode::F10 => Some(Key::F10),
        glutin::VirtualKeyCode::F11 => Some(Key::F11),
        glutin::VirtualKeyCode::F12 => Some(Key::F12),
        glutin::VirtualKeyCode::F13 => Some(Key::F13),
        glutin::VirtualKeyCode::F14 => Some(Key::F14),
        glutin::VirtualKeyCode::F15 => Some(Key::F15),
        glutin::VirtualKeyCode::Snapshot => Some(Key::PrintScreen),
        glutin::VirtualKeyCode::Scroll => Some(Key::ScrollLock),
        glutin::VirtualKeyCode::Pause => Some(Key::Pause),
        glutin::VirtualKeyCode::Insert => Some(Key::Insert),
        glutin::VirtualKeyCode::Home => Some(Key::Home),
        glutin::VirtualKeyCode::Delete => Some(Key::Delete),
        glutin::VirtualKeyCode::End => Some(Key::End),
        glutin::VirtualKeyCode::PageDown => Some(Key::PageDown),
        glutin::VirtualKeyCode::PageUp => Some(Key::PageUp),
        glutin::VirtualKeyCode::Left => Some(Key::Left),
        glutin::VirtualKeyCode::Up => Some(Key::Up),
        glutin::VirtualKeyCode::Right => Some(Key::Right),
        glutin::VirtualKeyCode::Down => Some(Key::Down),
        glutin::VirtualKeyCode::Back => Some(Key::Backspace),
        glutin::VirtualKeyCode::Return => Some(Key::Enter),
        glutin::VirtualKeyCode::Space => Some(Key::Space),
        glutin::VirtualKeyCode::Numlock => Some(Key::NumLock),
        glutin::VirtualKeyCode::Numpad0 => Some(Key::Numpad0),
        glutin::VirtualKeyCode::Numpad1 => Some(Key::Numpad1),
        glutin::VirtualKeyCode::Numpad2 => Some(Key::Numpad2),
        glutin::VirtualKeyCode::Numpad3 => Some(Key::Numpad3),
        glutin::VirtualKeyCode::Numpad4 => Some(Key::Numpad4),
        glutin::VirtualKeyCode::Numpad5 => Some(Key::Numpad5),
        glutin::VirtualKeyCode::Numpad6 => Some(Key::Numpad6),
        glutin::VirtualKeyCode::Numpad7 => Some(Key::Numpad7),
        glutin::VirtualKeyCode::Numpad8 => Some(Key::Numpad8),
        glutin::VirtualKeyCode::Numpad9 => Some(Key::Numpad9),
        glutin::VirtualKeyCode::Add => Some(Key::NumpadAdd),
        glutin::VirtualKeyCode::Apostrophe => Some(Key::Apostrophe),
        glutin::VirtualKeyCode::Backslash => Some(Key::Backslash),
        glutin::VirtualKeyCode::Capital => Some(Key::CapsLock),
        glutin::VirtualKeyCode::Comma => Some(Key::Comma),
        glutin::VirtualKeyCode::Decimal => Some(Key::NumpadDecimal),
        glutin::VirtualKeyCode::Divide => Some(Key::NumpadDivide),
        glutin::VirtualKeyCode::Equals => Some(Key::Equals),
        glutin::VirtualKeyCode::Grave => Some(Key::GraveAccent),
        glutin::VirtualKeyCode::LAlt => Some(Key::LeftAlt),
        glutin::VirtualKeyCode::LBracket => Some(Key::LeftBracket),
        glutin::VirtualKeyCode::LControl => Some(Key::LeftControl),
        glutin::VirtualKeyCode::LShift => Some(Key::LeftShift),
        glutin::VirtualKeyCode::LWin => Some(Key::LeftMeta),
        glutin::VirtualKeyCode::Minus => Some(Key::Minus),
        glutin::VirtualKeyCode::Multiply => Some(Key::NumpadMultiply),
        glutin::VirtualKeyCode::NumpadEnter => Some(Key::NumpadEnter),
        glutin::VirtualKeyCode::NumpadEquals => Some(Key::NumpadEquals),
        glutin::VirtualKeyCode::Period => Some(Key::Period),
        glutin::VirtualKeyCode::RAlt => Some(Key::RightAlt),
        glutin::VirtualKeyCode::RBracket => Some(Key::RightBracket),
        glutin::VirtualKeyCode::RControl => Some(Key::RightControl),
        glutin::VirtualKeyCode::RShift => Some(Key::RightShift),
        glutin::VirtualKeyCode::RWin => Some(Key::RightMeta),
        glutin::VirtualKeyCode::Semicolon => Some(Key::Semicolon),
        glutin::VirtualKeyCode::Slash => Some(Key::Slash),
        glutin::VirtualKeyCode::Subtract => Some(Key::NumpadSubtract),
        glutin::VirtualKeyCode::Tab => Some(Key::Tab),
        _ => None,
    }
}
