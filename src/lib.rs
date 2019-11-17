pub use gouache;

pub mod input;
pub mod backends;

use std::any::{Any, TypeId};
use std::cell::Cell;
use std::rc::Rc;

use gouache::{Color, Frame, Font, Glyph, Path, Vec2, Mat2x2};
use input::{Input, InputState};

macro_rules! id {
    () => { { static ID: u8 = 0; &ID as *const u8 as usize } }
}

pub trait Elem {
    fn apply(self, node: &mut Node, bounds: Bounds);
}

pub trait ElemList {
    fn apply_all(self, cursor: &mut Cursor, bounds: Bounds);

    fn chain<N: Elem>(self, next: N) -> Chain<Self, N> where Self: Sized {
        Chain(self, next)
    }
}

pub struct EmptyList;

impl ElemList for EmptyList {
    fn apply_all(self, cursor: &mut Cursor, bounds: Bounds) {}
}

pub struct Chain<A: ElemList, B: ElemList>(A, B);

impl<A: ElemList, B: ElemList> ElemList for Chain<A, B> {
    fn apply_all(self, cursor: &mut Cursor, bounds: Bounds) {
        self.0.apply_all(cursor, bounds);
        self.1.apply_all(cursor, bounds);
    }
}

impl<E: Elem> ElemList for E {
    fn apply_all(self, cursor: &mut Cursor, bounds: Bounds) {
        self.apply(cursor.add(), bounds);
    }
}

#[derive(Copy, Clone)]
pub struct Bounds {
    width: f32,
    height: f32,
}

impl Bounds {
    pub fn new(width: f32, height: f32) -> Bounds {
        Bounds { width, height }
    }
}

pub struct Node {
    tag: usize,
    offset: Vec2,
    size: Vec2,
    shape: Shape,
    children: Vec<Node>,

    hover: bool,
    dragging: bool,
    events: Vec<(Input, InputState)>,
}

impl Node {
    pub fn new() -> Node {
        Node::with_tag(id!())
    }

    fn with_tag(tag: usize) -> Node {
        Node {
            tag,
            offset: Vec2::new(0.0, 0.0),
            size: Vec2::new(0.0, 0.0),
            shape: Shape::Empty,
            children: Vec::new(),

            hover: false,
            dragging: false,
            events: Vec::new(),
        }
    }

    pub fn tag(&mut self, tag: usize) -> bool {
        if tag == self.tag {
            true
        } else {
            *self = Node::with_tag(tag);
            false
        }
    }

    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    pub fn set_shape(&mut self, shape: Shape) {
        self.shape = shape;
    }

    pub fn size(&self) -> (f32, f32) {
        (self.size.x, self.size.y)
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.size.x = width;
        self.size.y = height;
    }

    pub fn offset(&self) -> (f32, f32) {
        (self.offset.x, self.offset.y)
    }

    pub fn set_offset(&mut self, x: f32, y: f32) {
        self.offset.x = x;
        self.offset.y = y;
    }

    pub fn children(&self) -> &[Node] {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut [Node] {
        &mut self.children
    }

    pub fn edit_children(&mut self) -> Cursor {
        Cursor { node: self, index: 0 }
    }

    pub fn poll(&mut self) -> impl Iterator<Item=(Input, InputState)> {
        std::mem::replace(&mut self.events, Vec::new()).into_iter()
    }

    pub fn hover(&self) -> bool {
        self.hover
    }

    pub fn dragging(&self) -> bool {
        self.dragging
    }

    pub fn render(&self, frame: &mut Frame) {
        self.render_inner(frame, Vec2::new(0.0, 0.0))
    }

    fn render_inner(&self, frame: &mut Frame, offset: Vec2) {
        let offset = offset + self.offset;

        match &self.shape {
            Shape::Empty => {}
            Shape::Rect { position, dimensions, color } => {
                frame.draw_rect(offset + *position, *dimensions, Mat2x2::id(), *color);
            }
            Shape::Text { position, font, size, glyphs, color } => {
                frame.draw_text(font, *size, glyphs, offset + *position, Mat2x2::id(), *color);
            }
            Shape::Path { position, path, color } => {
                frame.draw_path(path, offset + *position, Mat2x2::id(), *color);
            }
        }

        for child in self.children.iter() {
            child.render_inner(frame, offset);
        }
    }

    pub fn input(&mut self, input: Input, input_state: &InputState) {
        self.input_inner(input, input_state, Vec2::new(0.0, 0.0));
    }

    fn input_inner(&mut self, input: Input, input_state: &InputState, offset: Vec2) {
        let offset = offset + self.offset;

        match input {
            Input::MouseMove => {
                let hover =
                    input_state.mouse_x >= offset.x && input_state.mouse_x < offset.x + self.size.x &&
                    input_state.mouse_y >= offset.y && input_state.mouse_y < offset.y + self.size.y;

                if self.dragging || self.hover || hover {
                    self.events.push((input, *input_state));
                    for child in self.children.iter_mut() {
                        child.input_inner(input, input_state, offset);
                    }
                }

                self.hover = hover;
            }
            Input::MouseDown(..) => {
                if self.hover {
                    self.dragging = true;

                    self.events.push((input, *input_state));
                    for child in self.children.iter_mut() {
                        child.input_inner(input, input_state, offset);
                    }
                }
            }
            Input::MouseUp(..) => {
                if self.hover || self.dragging {
                    self.events.push((input, *input_state));
                    for child in self.children.iter_mut() {
                        child.input_inner(input, input_state, offset);
                    }
                }

                self.dragging = false;
            }
            Input::Scroll(..) => {
                if self.hover || self.dragging {
                    self.events.push((input, *input_state));
                    for child in self.children.iter_mut() {
                        child.input_inner(input, input_state, offset);
                    }
                }
            }
            Input::KeyDown(..) | Input::KeyUp(..) | Input::Char(..) => {}
        }
    }
}

pub struct Cursor<'a> {
    node: &'a mut Node,
    index: usize,
}

impl<'a> Cursor<'a> {
    pub fn add(&mut self) -> &mut Node {
        assert!(self.index <= self.node.children.len());

        let child = if self.index == self.node.children.len() {
            self.node.children.push(Node::new());
            self.node.children.last_mut().unwrap()
        } else {
            &mut self.node.children[self.index]
        };

        self.index += 1;
        child
    }
}

impl<'a> Drop for Cursor<'a> {
    fn drop(&mut self) {
        self.node.children.truncate(self.index);
    }
}

pub enum Shape {
    Empty,
    Rect {
        position: Vec2,
        dimensions: Vec2,
        color: Color,
    },
    Text {
        position: Vec2,
        font: Rc<Font<'static>>,
        size: f32,
        glyphs: Vec<Glyph>,
        color: Color,
    },
    Path {
        position: Vec2,
        path: Rc<Path>,
        color: Color,
    },
}

pub struct Empty;

impl Elem for Empty {
    fn apply(self, node: &mut Node, bounds: Bounds) {}
}

pub struct Text {
    font: Rc<Font<'static>>,
    size: f32,
    text: &'static str,
}

impl Text {
    pub fn new(font: Rc<Font<'static>>, size: f32, text: &'static str) -> Text {
        Text { font, size, text }
    }
}

impl Elem for Text {
    fn apply(self, node: &mut Node, bounds: Bounds) {
        node.tag(id!());

        node.set_shape(Shape::Text {
            font: self.font.clone(),
            size: self.size,
            glyphs: self.font.layout(self.text, self.size),
            position: Vec2::new(0.0, 0.0),
            color: Color::rgba(1.0, 1.0, 1.0, 1.0),
        });

        let (width, height) = self.font.measure(self.text, self.size);
        node.set_size(width, height);
    }
}

pub struct Padding<C: Elem> {
    padding: f32,
    child: C,
}

impl<C: Elem> Padding<C> {
    pub fn new(padding: f32, child: C) -> Padding<C> {
        Padding { padding, child }
    }
}

impl<C: Elem> Elem for Padding<C> {
    fn apply(self, node: &mut Node, bounds: Bounds) {
        node.tag(id!());

        self.child.apply(node.edit_children().add(), Bounds::new(bounds.width - 2.0 * self.padding, bounds.height - 2.0 * self.padding));
        let mut child = &mut node.children_mut()[0];
        child.set_offset(self.padding, self.padding);
        let (width, height) = child.size();

        node.set_size(width + 2.0 * self.padding, height + 2.0 * self.padding);
    }
}

pub struct BackgroundColor<C: Elem> {
    color: Color,
    child: C,
}

impl<C: Elem> BackgroundColor<C> {
    pub fn new(color: Color, child: C) -> BackgroundColor<C> {
        BackgroundColor { color, child }
    }
}

impl<C: Elem> Elem for BackgroundColor<C> {
    fn apply(self, node: &mut Node, bounds: Bounds) {
        node.tag(id!());

        self.child.apply(node.edit_children().add(), bounds);
        let (width, height) = node.children()[0].size();

        node.set_shape(Shape::Rect {
            position: Vec2::new(0.0, 0.0),
            dimensions: Vec2::new(width, height),
            color: self.color,
        });
        node.set_size(width, height);
    }
}

pub struct Row<C: ElemList> {
    spacing: f32,
    children: C,
}

impl Row<EmptyList> {
    pub fn new(spacing: f32) -> Row<EmptyList> {
        Row { spacing, children: EmptyList }
    }
}

impl<C: ElemList> Row<C> {
    pub fn child<E: Elem>(self, child: E) -> Row<impl ElemList> {
        Row { spacing: self.spacing, children: self.children.chain(child) }
    }
}

impl<C: ElemList> Elem for Row<C> {
    fn apply(self, node: &mut Node, bounds: Bounds) {
        node.tag(id!());

        self.children.apply_all(&mut node.edit_children(), Bounds::new(std::f32::INFINITY, bounds.height));

        let mut x: f32 = 0.0;
        let mut height: f32 = 0.0;
        for child in node.children_mut() {
            child.set_offset(x, 0.0);
            let (child_width, child_height) = child.size();
            x += child_width + self.spacing;
            height = height.max(child_height);
        }

        node.set_size((x - self.spacing).max(0.0), height);
    }
}

pub struct Button<C: Elem, F> {
    child: C,
    on_click: F,
}

impl<C: Elem> Button<C, fn()> {
    pub fn new(child: C) -> Button<C, fn()> {
        Button { child, on_click: || {} }
    }
}

impl<C: Elem, F: FnMut()> Button<C, F> {
    pub fn on_click<G: FnMut()>(self, on_click: G) -> Button<C, G> {
        Button { child: self.child, on_click }
    }
}

impl<C: Elem, F: FnMut()> Elem for Button<C, F> {
    fn apply(mut self, node: &mut Node, bounds: Bounds) {
        node.tag(id!());

        for (input, input_state) in node.poll() {
            match input {
                Input::MouseUp(..) => {
                    if node.hover() { (self.on_click)(); }
                }
                _ => {},
            }
        }

        let color = if node.dragging() {
            Color::rgba(0.141, 0.44, 0.77, 1.0)
        } else if node.hover() {
            Color::rgba(0.54, 0.63, 0.71, 1.0)
        } else {
            Color::rgba(0.38, 0.42, 0.48, 1.0)
        };

        BackgroundColor::new(color, Padding::new(5.0, self.child))
            .apply(node.edit_children().add(), bounds);
        let (width, height) = node.children()[0].size();
        node.set_size(width, height);
    }
}
