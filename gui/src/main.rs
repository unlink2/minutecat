extern crate orbtk;

mod local;

use orbtk::prelude::*;

fn main() {
    Application::new()
        .localization(
            RonLocalization::create()
            .language("en_US")
            .dictionary("en_US", local::EN_US)
            .build()
        )
        .window(|ctx| {
            Window::new()
                .title("minutecat desktop")
                .size(420.0, 730.0)
                .child(TextBlock::new().text("HELLO_WORLD").build(ctx))
                .build(ctx)
        })
    .run();
}
