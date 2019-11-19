use std::rc::Rc;

use gouache::Font;
use casein::*;

fn main() {
    let font = Rc::new(Font::from_bytes(include_bytes!("../res/SourceSansPro-Regular.ttf")).unwrap());
    let rx = Receiver::new();

    backends::glutin::run(|| {
        for _ in rx.poll() {
            println!("click");
        }

        Row::new(5.0, (
            Button::new(Text::new(font.clone(), 14.0, "jackdaws love my"))
                .on_click({ let tx = rx.sender(); move || tx.send(()) }),
            Button::new(Text::new(font.clone(), 14.0, "big sphinx of quartz")),
        ))
    });
}
