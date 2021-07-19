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

        ctx.widget().set("path", path);
        ctx.widget()
            .set("logset", logset);
    }

    fn save(&mut self, ctx: &mut Context) {
        // TODO maybe do not panic here!
    }
}

impl State for MainViewState {
    fn init(&mut self, _registry: &mut Registry, ctx: &mut Context) {
        self.load(ctx);
    }

    fn update(&mut self, _registry: &mut Registry, ctx: &mut Context) {
        let logset = ctx.widget()
            .get_mut::<LogSet>("logset").len();
    }

    fn messages(
        &mut self,
        messages: MessageReader,
        _registry: &mut Registry,
        ctx: &mut Context) {
    }
}

widget!(MainView<MainViewState> {
});

impl Template for MainView {
    fn template(self, id: Entity, ctx: &mut BuildContext) -> Self {
        self.name("MainView")
    }
}


