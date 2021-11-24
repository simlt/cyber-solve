// https://users.rust-lang.org/t/reference-to-trait-objects-and-lifetime-issues-for-gui-code/67808
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=940063d43b110d8b41e8fdf180c2ce0b?

// window.rs
const WM_PAINT: u32 = 15u32;

trait Paintable {
    fn paint(&self);
}

struct Window<P: Paintable> {
    painter: P,
}

impl<P: Paintable> Window<P> {
    fn new(painter: P) -> Self {
        Self { painter }
    }
}

impl<P: Paintable> Window<P> {
    fn message_handler(&mut self, message: u32) {
        match message {
            WM_PAINT => { self.on_paint(); },
            _ => {}
        }
    }
    
    fn on_paint(&self) {
        self.painter.paint();
    }
}


// my-window.rs
struct MyPainter {
    data: u32,
}
impl Paintable for MyPainter {
    fn paint(&self) {
        println!("Hello MyWindow with data {}", self.data);
    }
}

struct MyWindow {
    window: Window<MyPainter>,
}

impl MyWindow {
    fn new() -> Self {
        let painter = MyPainter { data: 42 };
        let window = Window::new(painter);
        Self { window }
    }
}


fn main() {
    let mut my_wnd = MyWindow::new();
    
    // This is triggered from some other place
    my_wnd.window.message_handler(WM_PAINT);
    
    // I want to tell MyWindow to use its paint() when Window makes a paint, and
    // that would result in the following output: "Hello MyWindow with data 42"
}
