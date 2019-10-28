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

    let mut root = Text::new(font, 14.0, "jackdaws love my".to_string());

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
