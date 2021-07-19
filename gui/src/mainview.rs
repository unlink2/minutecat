use super::orbtk::prelude::*;
use super::minutecat::interface::command_line;
use super::minutecat::logset::LogSet;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Default, AsAny)]
pub struct MainViewState {
}

impl MainViewState {
    fn load(&mut self, ctx: &mut Context) {
        // TODO maybe do not panic here!
        // TODO find better way to load logset than
        // re-reading a file we already have!
        let (logset, path) = minutecat::interface::init_logset()
                .expect("Unable to read config!");

        ctx.widget()
            .set("path", path);
        ctx.widget()
            .set("logset", logset);
    }

    fn save(&mut self, ctx: &mut Context) {
        let path = ctx.widget().get_mut::<String>("path").clone();
        // TODO maybe do not panic here!
        ctx.widget()
            .get_mut::<LogSet>("logset")
            .to_file(&path)
            .expect("Unable to save config!");
    }
}

impl State for MainViewState {
    fn init(&mut self, _registry: &mut Registry, ctx: &mut Context) {
        self.load(ctx);
    }

    fn update(&mut self, _registry: &mut Registry, ctx: &mut Context) {
        TextBox::text_mut(&mut ctx.child("test")).push('!');
    }

    fn messages(
        &mut self,
        messages: MessageReader,
        _registry: &mut Registry,
        ctx: &mut Context) {
    }
}

widget!(MainView<MainViewState> {
    path: String,
    logset: LogSet
});

impl Template for MainView {
    fn template(self, id: Entity, ctx: &mut BuildContext) -> Self {
        self.name("MainView")
            .child(
                TextBox::new()
                    .text("MainView")
                    .id("test")
                    .build(ctx)
            )
    }
}


