use super::orbtk::prelude::*;

#[derive(Default, AsAny)]
pub struct LogListViewState {
}

impl State for LogListViewState {
    fn init(&mut self, _registry: &mut Registry, _ctx: &mut Context) {
    }

    fn messages(
        &mut self,
        _messages: MessageReader,
        _registry: &mut Registry,
        _ctx: &mut Context) {
    }
}

widget!(LogListView<LogListViewState> {
    text: String
});

impl Template for LogListView {
    fn template(self, _id: Entity, _ctx: &mut BuildContext) -> Self {
        self.name("LogListView")
    }
}


