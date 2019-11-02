pub use gouache;

pub mod input;

use std::any::Any;

use gouache::{Color, Frame, Font, Vec2, Mat2x2, TextLayout};
use input::{Input, InputState};

pub trait Elem: Any {
    fn handle(&mut self, input: Input, state: &InputState) -> Option<Response>;
    fn layout(&mut self, max_width: f32, max_height: f32);
    fn offset(&mut self, x: f32, y: f32);
    fn render(&mut self, frame: &mut Frame);
    fn rect(&self) -> Rect;
}

pub struct SingleElem {
    inside: bool,
    mouse_captured: bool,
    elem: Box<dyn Elem>,
}

impl SingleElem {
    pub fn new(elem: Box<dyn Elem>) -> SingleElem {
        SingleElem { inside: false, mouse_captured: false, elem }
    }

    pub fn handle(&mut self, input: Input, state: &InputState) -> Option<Response> {
        let response = match input {
            Input::MouseMove | Input::MouseEnter | Input::MouseLeave => {
                if self.elem.rect().contains(state.mouse_x, state.mouse_y) {
                    if self.inside {
                        self.elem.handle(Input::MouseMove, state)
                    } else {
                        self.inside = true;
                        self.elem.handle(Input::MouseEnter, state)
                    }
                } else if self.inside {
                    self.inside = false;
                    self.elem.handle(Input::MouseLeave, state)
                } else if self.mouse_captured {
                    self.elem.handle(Input::MouseMove, state)
                } else {
                    None
                }
            }
            Input::MouseDown(..) | Input::MouseUp(..) | Input::Scroll(..) => {
                if self.inside || self.mouse_captured {
                    self.elem.handle(input, state)
                } else {
                    None
                }
            }
            Input::KeyDown(..) | Input::KeyUp(..) | Input::Char(..) => {
                self.elem.handle(input, state)
            }
        };

        self.mouse_captured = response.map_or(false, |r| r.capture_mouse);

        response
    }

    pub fn layout(&mut self, max_width: f32, max_height: f32) {
        self.elem.layout(max_width, max_height);
    }

    pub fn offset(&mut self, x: f32, y: f32) {
        self.elem.offset(x, y);
    }

    pub fn render(&mut self, frame: &mut Frame) {
        self.elem.render(frame);
    }

    pub fn rect(&self) -> Rect {
        self.elem.rect()
    }
}

pub struct ElemList {
    mouse_captured: Option<usize>,
    elems: Vec<SingleElem>,
}

impl ElemList {
    fn new(elems: Vec<SingleElem>) -> ElemList {
        ElemList { mouse_captured: None, elems }
    }

    fn iter(&self) -> impl Iterator<Item=&SingleElem> {
        self.elems.iter()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item=&mut SingleElem> {
        self.elems.iter_mut()
    }

    fn handle(&mut self, input: Input, state: &InputState) -> Option<Response> {
        match input {
            Input::MouseMove | Input::MouseEnter | Input::MouseLeave |
            Input::MouseDown(..) | Input::MouseUp(..) | Input::Scroll(..) => {
                if let Some(i) = self.mouse_captured {
                    let response = self.elems[i].handle(input, state);
                    if !response.map_or(false, |r| r.capture_mouse) {
                        self.mouse_captured = None;
                    }
                    return response;
                }

                for (i, elem) in self.elems.iter_mut().enumerate() {
                    let response = elem.handle(input, state);
                    if let Some(response) = response {
                        if response.capture_mouse {
                            self.mouse_captured = Some(i);
                        }
                        return Some(response);
                    }
                }
                None
            }
            Input::KeyDown(..) | Input::KeyUp(..) | Input::Char(..) => {
                None
            }
        }
    }
}

#[derive(Copy, Clone)]
pub struct Response {
    capture_mouse: bool,
}

impl Response {
    pub fn capture_mouse(self) -> Self {
        Response { capture_mouse: true, ..self }
    }
}

impl Default for Response {
    fn default() -> Response {
        Response {
            capture_mouse: false,
        }
    }
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

pub struct Padding {
    padding: f32,
    rect: Rect,
    inside_child: bool,
    child: Box<dyn Elem>,
}

impl Padding {
    pub fn new(padding: f32, child: Box<dyn Elem>) -> Padding {
        Padding { padding, child, inside_child: false, rect: Rect::new(0.0, 0.0, 0.0, 0.0) }
    }
}

impl Elem for Padding {
    fn handle(&mut self, input: Input, state: &InputState) -> Option<Response> {
        self.child.handle(input, state)
    }

    fn layout(&mut self, max_width: f32, max_height: f32) {
        self.child.layout(max_width - 2.0 * self.padding, max_height - 2.0 * self.padding);
        let rect = self.child.rect();
        self.rect.width = rect.width + 2.0 * self.padding;
        self.rect.height = rect.height + 2.0 * self.padding;
    }

    fn offset(&mut self, x: f32, y: f32) {
        self.rect.x = x;
        self.rect.y = y;
        self.child.offset(self.padding + x, self.padding + y);
    }

    fn render(&mut self, frame: &mut Frame) {
        self.child.render(frame);
    }

    fn rect(&self) -> Rect {
        self.rect
    }
}

pub struct BackgroundColor {
    color: Color,
    child: Box<dyn Elem>,
}

impl BackgroundColor {
    pub fn new(color: Color, child: Box<dyn Elem>) -> BackgroundColor {
        BackgroundColor { color, child }
    }
}

impl Elem for BackgroundColor {
    fn handle(&mut self, input: Input, state: &InputState) -> Option<Response> {
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

impl Elem for Text {
    fn handle(&mut self, input: Input, state: &InputState) -> Option<Response> {
        None
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

pub struct Button {
    inside: bool,
    down: bool,
    contents: BackgroundColor,
}

impl Button {
    pub fn new(child: Box<dyn Elem>) -> Button {
        Button { inside: false, down: false, contents: BackgroundColor::new(Color::rgba(0.38, 0.42, 0.48, 1.0), child) }
    }
}

impl Elem for Button {
    fn handle(&mut self, input: Input, state: &InputState) -> Option<Response> {
        let rect = self.contents.rect();

        match input {
            Input::MouseEnter => {
                self.inside = true;
            }
            Input::MouseLeave => {
                self.inside = false;
            }
            Input::MouseDown(..) => {
                if self.inside {
                    self.down = true;
                }
            }
            Input::MouseUp(..) => {
                if self.inside && self.down {
                    println!("click");
                }
                self.down = false;
            }
            _ => {
                self.contents.handle(input, state);
            }
        }

        self.contents.color = if self.down {
            Color::rgba(0.141, 0.44, 0.77, 1.0)
        } else if self.inside {
            Color::rgba(0.54, 0.63, 0.71, 1.0)
        } else {
            Color::rgba(0.38, 0.42, 0.48, 1.0)
        };

        if self.down {
            Some(Response::default().capture_mouse())
        } else {
            None
        }
    }

    fn layout(&mut self, max_width: f32, max_height: f32) {
        self.contents.layout(max_width, max_height);
    }

    fn offset(&mut self, x: f32, y: f32) {
        self.contents.offset(x, y);
    }

    fn render(&mut self, frame: &mut Frame) {
        self.contents.render(frame);
    }

    fn rect(&self) -> Rect {
        self.contents.rect()
    }
}
