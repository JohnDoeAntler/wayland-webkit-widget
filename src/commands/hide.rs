use super::Command;
use crate::{
    cli::{AppState, QueryArgs},
    utils::widget_filter,
};

pub struct HideCommand<'a> {
    pub config: &'a mut AppState,
    pub query: QueryArgs,
}

impl<'a> HideCommand<'a> {
    pub fn new(config: &'a mut AppState, query: QueryArgs) -> Self {
        Self { config, query }
    }
}

impl Command for HideCommand<'_> {
    fn execute(&mut self) -> String {
        self.config
            .widgets
            .iter()
            .filter(|w| widget_filter(w, self.query.to_owned()))
            .map(|e| {
                e.hide();
                e.id.to_owned()
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}
