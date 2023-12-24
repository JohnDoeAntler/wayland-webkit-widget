use super::Command;
use crate::{
    cli::{AppState, QueryArgs},
    utils::widget_filter,
};

pub struct ReloadCommand<'a> {
    pub config: &'a mut AppState,
    pub query: QueryArgs,
}

impl<'a> ReloadCommand<'a> {
    pub fn new(config: &'a mut AppState, query: QueryArgs) -> Self {
        Self { config, query }
    }
}

impl Command for ReloadCommand<'_> {
    fn execute(&mut self) -> String {
        self.config
            .widgets
            .iter()
            .filter(|w| widget_filter(w, self.query.to_owned()))
            .map(|e| {
                e.reload();
                e.id.to_owned()
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}
