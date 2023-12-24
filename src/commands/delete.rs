use super::Command;
use crate::{
    cli::{AppState, QueryArgs},
    utils::widget_filter,
};

pub struct DeleteCommand<'a> {
    pub config: &'a mut AppState,
    pub query: QueryArgs,
}

impl<'a> DeleteCommand<'a> {
    pub fn new(config: &'a mut AppState, query: QueryArgs) -> Self {
        Self { config, query }
    }
}

impl Command for DeleteCommand<'_> {
    fn execute(&mut self) -> String {
        let mut ret = String::new();

        self.config
            .widgets
            .retain(|w| {
                if widget_filter(w, self.query.to_owned()) {
                    w.close();
                    ret.push_str(&w.id);
                    false
                } else {
                    true
                }
            });

        ret.to_string()
    }
}

