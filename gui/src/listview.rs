use super::orbtk::prelude::*;

#[derive(Default, AsAny)]
pub struct LogListViewState {
}

impl State for LogListViewState {
    fn messages(
        &mut self,
        messages: MessageReader,
        _registry: &mut Registry,
        ctx: &mut Context) {
    }
}

widget!(LogListView<LogListViewState> {
    text: String
});

impl Template for LogListView {
    fn template(self, id: Entity, ctx: &mut BuildContext) -> Self {
        self.name("LogListView")
    }
}


