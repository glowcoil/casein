pub use gouache;

pub mod input;

use std::any::Any;

use gouache::{Color, Frame, Font, Vec2, Mat2x2, TextLayout};
use input::{Input, InputState};

pub trait View: Any {
    fn handle(&mut self, input: Input, state: &InputState) -> Response;
    fn layout(&mut self, max_width: f32, max_height: f32);
    fn offset(&mut self, x: f32, y: f32);
    fn render(&mut self, frame: &mut Frame);
    fn rect(&self) -> Rect;
}

pub enum Response {
    None,
}

#[derive(Copy, Clone)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Rect {
        Rect { x, y, width, height }
    }

    fn contains(&self, x: f32, y: f32) -> bool {
        self.x <= x && x < self.x + self.width &&
        self.y <= y && y < self.y + self.height
    }
}

pub struct BackgroundColor {
    color: Color,
    child: Box<dyn View>,
}

impl BackgroundColor {
    pub fn new(color: Color, child: Box<dyn View>) -> BackgroundColor {
        BackgroundColor { color, child }
    }
}

impl View for BackgroundColor {
    fn handle(&mut self, input: Input, state: &InputState) -> Response {
        self.child.handle(input, state)
    }

    fn layout(&mut self, max_width: f32, max_height: f32) {
        self.child.layout(max_width, max_height);
    }

    fn offset(&mut self, x: f32, y: f32) {
        self.child.offset(x, y);
    }

    fn render(&mut self, frame: &mut Frame) {
        let rect = self.child.rect();
        frame.draw_rect(rect.x, rect.y, rect.width, rect.height, Mat2x2::id(), self.color);

        self.child.render(frame);
    }

    fn rect(&self) -> Rect {
        self.child.rect()
    }
}

pub struct Text {
    font: Font<'static>,
    size: f32,
    text: String,
    rect: Rect,
    layout: TextLayout,
}

impl Text {
    pub fn new(font: Font<'static>, size: f32, text: String) -> Text {
        Text { font, size, text, rect: Rect::new(0.0, 0.0, 0.0, 0.0), layout: TextLayout::empty() }
    }
}

impl View for Text {
    fn handle(&mut self, input: Input, state: &InputState) -> Response {
        Response::None
    }

    fn layout(&mut self, max_width: f32, max_height: f32) {
        self.layout = self.font.layout(&self.text, self.size);
        self.rect.width = self.layout.width;
        self.rect.height = self.layout.height;
    }

    fn offset(&mut self, x: f32, y: f32) {
        self.rect.x = x;
        self.rect.y = y;
    }

    fn render(&mut self, frame: &mut Frame) {
        frame.draw_text(&self.font, &self.layout, Vec2::new(self.rect.x, self.rect.y), Mat2x2::id(), Color::rgba(1.0, 1.0, 1.0, 1.0));
    }

    fn rect(&self) -> Rect {
        self.rect
    }
}
