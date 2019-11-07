pub use gouache;

pub mod input;
pub mod backends;

use std::any::{Any, TypeId};
use std::cell::Cell;
use std::rc::Rc;

use gouache::{Color, Frame, Font, Vec2, Mat2x2, TextLayout};
use input::{Input, InputState};

pub trait Template {
    fn install(self, elem: &mut Node);
}

pub struct Id;

impl Template for Id {
    fn install(self, elem: &mut Node) {}
}

impl<F: FnOnce(&mut Node)> Template for F {
    fn install(self, elem: &mut Node) {
        self(elem)
    }
}

pub trait ListTemplate {
    fn install(self, list: Cursor);
}

impl<F: FnOnce(Cursor)> ListTemplate for F {
    fn install(self, elem: Cursor) {
        self(elem)
    }
}

#[macro_export]
macro_rules! children {
    ($($child:expr),*) => (
        |mut cursor: Cursor| {
            $($child.install(cursor.next());)*
        }
    );
    ($($child:expr,)*) => ($crate::children![$($child),*])
}

pub trait Elem: Any {
    fn handle(&mut self, input: Input, state: &InputState) -> Option<Response>;
    fn layout(&mut self, max_width: f32, max_height: f32);
    fn offset(&mut self, x: f32, y: f32);
    fn render(&mut self, frame: &mut Frame);
    fn rect(&self) -> Rect;

    fn get_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

pub struct Node {
    inside: bool,
    mouse_captured: bool,
    elem: Box<dyn Elem>,
}

impl Node {
    pub fn new() -> Node {
        Node { inside: false, mouse_captured: false, elem: Box::new(Empty) }
    }

    pub fn get<E: Elem>(&self) -> Option<&E> {
        if self.elem.get_type_id() == TypeId::of::<E>() {
            Some(unsafe { &*(&*self.elem as *const dyn Elem as *const E) })
        } else {
            None
        }
    }

    pub fn get_mut<E: Elem>(&mut self) -> Option<&mut E> {
        if self.elem.get_type_id() == TypeId::of::<E>() {
            Some(unsafe { &mut *(&mut *self.elem as *mut dyn Elem as *mut E) })
        } else {
            None
        }
    }

    pub fn place<E: Elem>(&mut self, elem: E) {
        *self = Node { inside: false, mouse_captured: false, elem: Box::new(elem) };
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

pub struct NodeList {
    mouse_captured: Option<usize>,
    elems: Vec<Node>,
}

impl NodeList {
    pub fn new() -> NodeList {
        NodeList { mouse_captured: None, elems: Vec::new() }
    }

    pub fn cursor<'a>(&'a mut self) -> Cursor<'a> {
        Cursor::new(self)
    }

    pub fn iter(&self) -> impl Iterator<Item=&Node> {
        self.elems.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut Node> {
        self.elems.iter_mut()
    }

    pub fn handle(&mut self, input: Input, state: &InputState) -> Option<Response> {
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

pub struct Cursor<'a> {
    i: usize,
    list: &'a mut NodeList,
}

impl<'a> Cursor<'a> {
    fn new(list: &mut NodeList) -> Cursor {
        Cursor { i: 0, list }
    }

    pub fn next(&mut self) -> &mut Node {
        let result = if self.i < self.list.elems.len() {
            self.list.elems.get_mut(self.i).unwrap()
        } else {
            self.list.elems.push(Node::new());
            self.list.elems.last_mut().unwrap()
        };
        self.i += 1;
        result
    }
}

impl<'a> Drop for Cursor<'a> {
    fn drop(&mut self) {
        self.list.elems.truncate(self.i);
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

pub struct Receiver<T>(Rc<Cell<Vec<T>>>);

impl<T> Receiver<T> {
    pub fn new() -> Receiver<T> {
        Receiver(Rc::new(Cell::new(Vec::new())))
    }

    pub fn sender(&self) -> Sender<T> {
        Sender(self.0.clone())
    }

    pub fn drain(&mut self) -> impl Iterator<Item=T> {
        self.0.replace(Vec::new()).into_iter()
    }
}

pub struct Sender<T>(Rc<Cell<Vec<T>>>);

impl<T> Sender<T> {
    pub fn send(&self, value: T) {
        let mut queue = self.0.replace(Vec::new());
        queue.push(value);
        self.0.set(queue);
    }
}

struct Empty;

impl Elem for Empty {
    fn handle(&mut self, input: Input, state: &InputState) -> Option<Response> {
        None
    }

    fn layout(&mut self, max_width: f32, max_height: f32) {}

    fn offset(&mut self, x: f32, y: f32) {}

    fn render(&mut self, frame: &mut Frame) {}

    fn rect(&self) -> Rect {
        Rect::new(0.0, 0.0, 0.0, 0.0)
    }
}

pub struct Row {
    spacing: f32,
    children: NodeList,
    rect: Rect,
}

impl Row {
    pub fn new(spacing: f32, children: impl ListTemplate) -> impl Template {
        move |node: &mut Node| {
            if let Some(elem) = node.get_mut::<Row>() {
                elem.spacing = spacing;
                children.install(elem.children.cursor());
            } else {
                let mut row = Row { spacing, children: NodeList::new(), rect: Rect::new(0.0, 0.0, 0.0, 0.0) };
                children.install(row.children.cursor());
                node.place(row);
            }
        }
    }
}

impl Elem for Row {
    fn handle(&mut self, input: Input, state: &InputState) -> Option<Response> {
        self.children.handle(input, state)
    }

    fn layout(&mut self, max_width: f32, max_height: f32) {
        let mut width: f32 = 0.0;
        let mut height: f32 = 0.0;
        for child in self.children.iter_mut() {
            child.layout(std::f32::INFINITY, max_height);
            let rect = child.rect();
            width += rect.width + self.spacing;
            height = height.max(rect.height);
        }

        self.rect.width = width;
        self.rect.height = height;
    }

    fn offset(&mut self, x: f32, y: f32) {
        self.rect.x = x;
        self.rect.y = y;

        let mut x = x;
        for child in self.children.iter_mut() {
            child.offset(x, y);
            x += child.rect().width + self.spacing;
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        for child in self.children.iter_mut() {
            child.render(frame);
        }
    }

    fn rect(&self) -> Rect {
        self.rect
    }
}

pub struct Padding {
    padding: f32,
    rect: Rect,
    inside_child: bool,
    child: Node,
}

impl Padding {
    pub fn new(padding: f32, child: impl Template) -> impl Template {
        move |node: &mut Node| {
            if let Some(elem) = node.get_mut::<Padding>() {
                elem.padding = padding;
                child.install(&mut elem.child);
            } else {
                let mut child_node = Node::new();
                child.install(&mut child_node);
                node.place(Padding { padding, child: child_node, inside_child: false, rect: Rect::new(0.0, 0.0, 0.0, 0.0) });
            }
        }
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
    child: Node,
}

impl BackgroundColor {
    pub fn new(color: Color, child: impl Template) -> impl Template {
        move |node: &mut Node| {
            if let Some(elem) = node.get_mut::<BackgroundColor>() {
                elem.color = color;
                child.install(&mut elem.child);
            } else {
                let mut child_node = Node::new();
                child.install(&mut child_node);
                node.place(BackgroundColor { color, child: child_node });
            }
        }
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
    font: Rc<Font<'static>>,
    size: f32,
    text: String,
    rect: Rect,
    layout: TextLayout,
}

impl Text {
    pub fn new(font: Rc<Font<'static>>, size: f32, text: String) -> impl Template {
        move |node: &mut Node| {
            if let Some(elem) = node.get_mut::<Text>() {
                elem.font = font;
                elem.size = size;
                elem.text = text;
            } else {
                node.place(Text {
                    font, size, text, rect: Rect::new(0.0, 0.0, 0.0, 0.0), layout: TextLayout::empty(),
                });
            }
        }
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

pub struct ButtonTemplate<T: Template, F: Fn() + 'static> {
    child: T,
    on_click: F,
}

impl<T: Template, F: Fn() + 'static> ButtonTemplate<T, F> {
    pub fn on_click<G: Fn()>(self, on_click: G) -> ButtonTemplate<T, G> {
        ButtonTemplate { child: self.child, on_click }
    }
}

impl<T: Template, F: Fn() + 'static> Template for ButtonTemplate<T, F> {
    fn install(self, node: &mut Node) {
        if let Some(button) = node.get_mut::<Button>() {
            button.on_click = Box::new(self.on_click);
            button.update(self.child);
        } else {
            let mut button = Button {
                inside: false,
                down: false,
                on_click: Box::new(self.on_click),
                contents: Node::new()
            };
            button.update(self.child);
            node.place(button);
        }
    }
}

pub struct Button {
    inside: bool,
    down: bool,
    on_click: Box<dyn Fn()>,
    contents: Node,
}

impl Button {
    pub fn new<T: Template>(child: T) -> ButtonTemplate<T, impl Fn()> {
        ButtonTemplate { child, on_click: Box::new(|| {}) }
    }

    fn update(&mut self, child: impl Template) {
        let color = if self.down {
            Color::rgba(0.141, 0.44, 0.77, 1.0)
        } else if self.inside {
            Color::rgba(0.54, 0.63, 0.71, 1.0)
        } else {
            Color::rgba(0.38, 0.42, 0.48, 1.0)
        };

        BackgroundColor::new(color, child).install(&mut self.contents);
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
                    (self.on_click)();
                }
                self.down = false;
            }
            _ => {
                self.contents.handle(input, state);
            }
        }

        self.update(Id);

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
