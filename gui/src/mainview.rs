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

    fn init_list(&mut self, ctx: &mut Context) {
        let stack = ctx.entity_of_child("logfile_list").unwrap();

        let build_ctx = &mut ctx.build_context();
        let new_child = TextBlock::new().text("Hello world!\nMulti line").build(build_ctx);
        build_ctx.append_child(stack, new_child);

    }
}

impl State for MainViewState {
    fn init(&mut self, _registry: &mut Registry, ctx: &mut Context) {
        self.load(ctx);
        self.init_list(ctx);
    }

    fn update(&mut self, _registry: &mut Registry, ctx: &mut Context) {
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
                Grid::new()
                .child(
                    Container::new()
                    .style("toolbar")
                    .attach(Grid::column_span(3))
                    .child(
                        Stack::new()
                        .orientation("horizontal")
                        .id("logfile_list")
                        .build(ctx)
                    )
                    .build(ctx)
                )
                .build(ctx)
            )
    }
}


