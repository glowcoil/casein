pub use gouache;

pub mod input;
pub mod backends;

use std::any::{Any, TypeId};
use std::cell::Cell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use gouache::{Color, Frame, Font, Glyph, Path, Vec2, Mat2x2};
use input::{Input, InputState, MouseButton, Key};

#[macro_export]
macro_rules! id {
    () => { { static ID: u8 = 0; &ID as *const u8 as usize } }
}

pub trait Elem {
    fn apply(self, node: &mut Node, bounds: Bounds);
}

pub trait ElemList {
    fn apply_all(self, cursor: &mut Cursor, bounds: Bounds);

    fn chain<N: ElemList>(self, next: N) -> Chain<Self, N> where Self: Sized {
        Chain(self, next)
    }
}

impl ElemList for () {
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

macro_rules! tuple_elem_list {
    ( $(($i:tt) $E:ident),+ ) => {
        impl<$($E: Elem),+> ElemList for ($($E,)+) {
            fn apply_all(self, cursor: &mut Cursor, bounds: Bounds) {
                $(self.$i.apply(cursor.add(), bounds);)+
            }
        }
    }
}

tuple_elem_list!((0) A);
tuple_elem_list!((0) A, (1) B);
tuple_elem_list!((0) A, (1) B, (2) C);
tuple_elem_list!((0) A, (1) B, (2) C, (3) D);
tuple_elem_list!((0) A, (1) B, (2) C, (3) D, (4) E);
tuple_elem_list!((0) A, (1) B, (2) C, (3) D, (4) E, (5) F);
tuple_elem_list!((0) A, (1) B, (2) C, (3) D, (4) E, (5) F, (6) G);
tuple_elem_list!((0) A, (1) B, (2) C, (3) D, (4) E, (5) F, (6) G, (7) H);
tuple_elem_list!((0) A, (1) B, (2) C, (3) D, (4) E, (5) F, (6) G, (7) H, (8) I);
tuple_elem_list!((0) A, (1) B, (2) C, (3) D, (4) E, (5) F, (6) G, (7) H, (8) I, (9) J);
tuple_elem_list!((0) A, (1) B, (2) C, (3) D, (4) E, (5) F, (6) G, (7) H, (8) I, (9) J, (10) K);
tuple_elem_list!((0) A, (1) B, (2) C, (3) D, (4) E, (5) F, (6) G, (7) H, (8) I, (9) J, (10) K, (11) L);

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
    handlers: Handlers,
    state: Box<dyn Any>,
}

struct Handlers {
    on_mouse_move: Option<Box<dyn Fn(&InputState)>>,
    on_mouse_down: Option<Box<dyn Fn(MouseButton, &InputState)>>,
    on_mouse_up: Option<Box<dyn Fn(MouseButton, &InputState)>>,
    on_scroll: Option<Box<dyn Fn(f32, f32, &InputState)>>,
    on_key_down: Option<Box<dyn Fn(Key, &InputState)>>,
    on_key_up: Option<Box<dyn Fn(Key, &InputState)>>,
    on_char: Option<Box<dyn Fn(char, &InputState)>>,
}

impl Default for Handlers {
    fn default() -> Handlers {
        Handlers {
            on_mouse_move: None,
            on_mouse_down: None,
            on_mouse_up: None,
            on_scroll: None,
            on_key_down: None,
            on_key_up: None,
            on_char: None,
        }
    }
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
            handlers: Handlers::default(),
            state: Box::new(()),
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

    pub fn state<T: 'static>(&mut self, default: impl Fn() -> T) -> &mut T {
        if (*self.state).type_id() != TypeId::of::<T>() {
            self.state = Box::new(default());
        }

        self.state.downcast_mut::<T>().unwrap()
    }

    pub fn on_mouse_move(&mut self, f: impl Fn(&InputState) + 'static) {
        self.handlers.on_mouse_move = Some(Box::new(f));
    }

    pub fn on_mouse_down(&mut self, f: impl Fn(MouseButton, &InputState) + 'static) {
        self.handlers.on_mouse_down = Some(Box::new(f));
    }

    pub fn on_mouse_up(&mut self, f: impl Fn(MouseButton, &InputState) + 'static) {
        self.handlers.on_mouse_up = Some(Box::new(f));
    }

    pub fn on_scroll(&mut self, f: impl Fn(f32, f32, &InputState) + 'static) {
        self.handlers.on_scroll = Some(Box::new(f));
    }

    pub fn on_key_down(&mut self, f: impl Fn(Key, &InputState) + 'static) {
        self.handlers.on_key_down = Some(Box::new(f));
    }

    pub fn on_key_up(&mut self, f: impl Fn(Key, &InputState) + 'static) {
        self.handlers.on_key_up = Some(Box::new(f));
    }

    pub fn on_char(&mut self, f: impl Fn(char, &InputState) + 'static) {
        self.handlers.on_char = Some(Box::new(f));
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
                    if let Some(ref on_mouse_move) = self.handlers.on_mouse_move {
                        on_mouse_move(input_state);
                    }
                    for child in self.children.iter_mut() {
                        child.input_inner(input, input_state, offset);
                    }
                }

                self.hover = hover;
            }
            Input::MouseDown(button) => {
                if self.hover {
                    self.dragging = true;

                    if let Some(ref on_mouse_down) = self.handlers.on_mouse_down {
                        on_mouse_down(button, input_state);
                    }
                    for child in self.children.iter_mut() {
                        child.input_inner(input, input_state, offset);
                    }
                }
            }
            Input::MouseUp(button) => {
                if self.hover || self.dragging {
                    if let Some(ref on_mouse_up) = self.handlers.on_mouse_up {
                        on_mouse_up(button, input_state);
                    }
                    for child in self.children.iter_mut() {
                        child.input_inner(input, input_state, offset);
                    }
                }

                self.dragging = false;
            }
            Input::Scroll(dx, dy) => {
                if self.hover || self.dragging {
                    if let Some(ref on_scroll) = self.handlers.on_scroll {
                        on_scroll(dx, dy, input_state);
                    }
                    for child in self.children.iter_mut() {
                        child.input_inner(input, input_state, offset);
                    }
                }
            }
            Input::KeyDown(..) | Input::KeyUp(..) | Input::Char(..) => {}
        }
    }
}

pub struct Receiver<T> {
    queue: Rc<Cell<Vec<T>>>,
}

impl<T: 'static> Receiver<T> {
    pub fn new() -> Receiver<T> {
        Receiver { queue: Rc::new(Cell::new(Vec::new())) }
    }

    pub fn sender(&self) -> Sender<T> {
        Sender { queue: self.queue.clone() }
    }

    pub fn poll(&self) -> impl Iterator<Item=T> {
        self.queue.replace(Vec::new()).into_iter()
    }
}

#[derive(Clone)]
pub struct Sender<T> {
    queue: Rc<Cell<Vec<T>>>,
}

impl<T: 'static> Sender<T> {
    pub fn send(&self, value: T) {
        let mut queue = self.queue.replace(Vec::new());
        queue.push(value);
        self.queue.set(queue);
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

impl<C: ElemList> Row<C> {
    pub fn new(spacing: f32, children: C) -> Row<C> {
        Row { spacing, children }
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

pub struct Col<C: ElemList> {
    spacing: f32,
    children: C,
}

impl<C: ElemList> Col<C> {
    pub fn new(spacing: f32, children: C) -> Col<C> {
        Col { spacing, children }
    }
}

impl<C: ElemList> Elem for Col<C> {
    fn apply(self, node: &mut Node, bounds: Bounds) {
        node.tag(id!());

        self.children.apply_all(&mut node.edit_children(), Bounds::new(std::f32::INFINITY, bounds.height));

        let mut y: f32 = 0.0;
        let mut width: f32 = 0.0;
        for child in node.children_mut() {
            child.set_offset(0.0, y);
            let (child_width, child_height) = child.size();
            y += child_height + self.spacing;
            width = width.max(child_width);
        }

        node.set_size(width, (y - self.spacing).max(0.0));
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

impl<C: Elem, F: Fn()> Button<C, F> {
    pub fn on_click<G: Fn()>(self, on_click: G) -> Button<C, G> {
        Button { child: self.child, on_click }
    }
}

impl<C: Elem, F: Fn() + 'static> Elem for Button<C, F> {
    fn apply(mut self, node: &mut Node, bounds: Bounds) {
        node.tag(id!());

        let on_click = self.on_click;
        node.on_mouse_up(move |button, input_state| {
            if button == MouseButton::Left {
                on_click();
            }
        });

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

pub struct Scrollable<C: Elem> {
    child: C,
}

impl<C: Elem> Scrollable<C> {
    pub fn new(child: C) -> Scrollable<C> {
        Scrollable { child }
    }
}

impl<C: Elem> Elem for Scrollable<C> {
    fn apply(mut self, node: &mut Node, bounds: Bounds) {
        node.tag(id!());

        self.child.apply(node.edit_children().add(), Bounds::new(bounds.width, std::f32::INFINITY));
        let (width, height) = node.children()[0].size();

        struct ScrollState {
            offset: f32,
            rx: Receiver<f32>,
        }
        let mut state = node.state(|| ScrollState { offset: 0.0, rx: Receiver::new() });
        for dy in state.rx.poll() {
            state.offset = (state.offset - dy).min(height - bounds.height).max(0.0);
        }

        let tx = state.rx.sender();
        let offset = state.offset;
        node.on_scroll(move |dx, dy, input_state| {
            tx.send(dy);
        });

        node.children_mut()[0].set_offset(0.0, -offset);
        node.set_size(width, height);
    }
}
