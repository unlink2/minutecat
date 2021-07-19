extern crate orbtk;
extern crate minutecat;

mod local;
mod listview;
mod logs;
mod mainview;

use mainview::MainView;
use minutecat::interface::command_line;
use minutecat::error::BoxResult;
use orbtk::prelude::*;

fn main() -> BoxResult<()> {
    let _interface = command_line()?;

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
                .child(MainView::new().build(ctx))
                .build(ctx)
        })
    .run();

    Ok(())
}
