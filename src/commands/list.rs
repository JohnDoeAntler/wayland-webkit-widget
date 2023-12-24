use super::Command;
use crate::cli::{AppState, QueryArgs};

pub struct ListCommand<'a> {
    pub query: QueryArgs,
    pub config: &'a mut AppState,
}

impl<'a> ListCommand<'a> {
    pub fn new(config: &'a mut AppState, query: QueryArgs) -> Self {
        Self { query, config }
    }
}

impl Command for ListCommand<'_> {
    fn execute(&mut self) -> String {
        let mut ret = String::new();

        for w in self.config.widgets.iter() {
            ret.push_str(format!("id: {}", w.id).as_str());
        }

        ret
    }
}
