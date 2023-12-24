use super::Command;
use crate::{
    cli::{AppState, QueryArgs, WidgetMetadataArgs},
    utils::widget_filter,
};

pub struct UpdateCommand<'a> {
    pub config: &'a mut AppState,
    pub query: QueryArgs,
    pub metadata: WidgetMetadataArgs,
}

impl<'a> UpdateCommand<'a> {
    pub fn new(config: &'a mut AppState, query: QueryArgs, metadata: WidgetMetadataArgs) -> Self {
        Self {
            config,
            query,
            metadata,
        }
    }
}

impl Command for UpdateCommand<'_> {
    fn execute(&mut self) -> String {
        self.config
            .widgets
            .iter_mut()
            .filter(|w| widget_filter(w, self.query.to_owned()))
            .map(|e| {
                e.update(self.metadata.to_owned());
                e.id.to_owned()
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}
