use std::rc::Rc;

use gouache::Font;
use casein::*;

fn main() {
    let font = Rc::new(Font::from_bytes(include_bytes!("../res/SourceSansPro-Regular.ttf")).unwrap());

    backends::glutin::run(||
        Row::new(5.0)
            .child(
                Button::new(Text::new(font.clone(), 14.0, "jackdaws love my"))
                    .on_click(|| println!("click"))
            )
            .child(Button::new(Text::new(font.clone(), 14.0, "big sphinx of quartz")))
    );
}
