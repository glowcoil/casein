use std::rc::Rc;

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

    let mut input_state = InputState::default();

    let font = Rc::new(Font::from_bytes(include_bytes!("../res/SourceSansPro-Regular.ttf")).unwrap());

    let mut receiver = Receiver::new();
    let sender = receiver.sender();

    let mut root = Node::new();
    Row::new(
        5.0,
        children![
            Button::new(
                Padding::new(
                    5.0,
                    Text::new(font.clone(), 14.0, "jackdaws love my".to_string()),
                ),
            ).on_click(move || { sender.send(()) }),
            Button::new(
                Padding::new(
                    5.0,
                    Text::new(font.clone(), 14.0, "big sphinx of quartz".to_string()),
                ),
            ),
        ],
    ).install(&mut root);

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

        let elapsed = now.elapsed();
        if elapsed < FRAME {
            std::thread::sleep(FRAME - elapsed);
        }
        now = std::time::Instant::now();

        events_loop.poll_events(|event| {
            match event {
                glutin::Event::WindowEvent { ref event, .. } => {
                    match event {
                        glutin::WindowEvent::CloseRequested => running = false,
                        glutin::WindowEvent::Resized(logical_size) => {
                            let dpi_factor = context.window().get_hidpi_factor();
                            context.resize(logical_size.to_physical(dpi_factor));
                        }
                        _ => {}
                    }
                }
                _ => {}
            }

            if let Some(input) = backends::glutin::process_event(event, &mut input_state) {
                root.handle(input, &input_state);
            }
        });

        for _ in receiver.drain() {
            println!("click");
        }
    }
}
