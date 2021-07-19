use super::orbtk::prelude::*;
use super::minutecat::interface::command_line;
use super::minutecat::logset::LogSet;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Default, AsAny)]
pub struct MainViewState {
}

impl State for MainViewState {
    fn init(&mut self, _registry: &mut Registry, ctx: &mut Context) {
        // TODO maybe do not panic here!
        // TODO find better way to load logset than
        // re-reading a file we already have!
        ctx.widget()
            .set("logset", minutecat::interface::init_logset()
                .expect("Unable to read config!").0);
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
    text: String,
    logset: LogSet
});

impl Template for MainView {
    fn template(self, id: Entity, ctx: &mut BuildContext) -> Self {
        self.name("MainView")
    }
}


