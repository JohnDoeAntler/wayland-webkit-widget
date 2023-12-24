use super::Command;
use crate::{
    cli::{AppState, WidgetMetadataArgs},
    widget::Widget,
};

pub struct CreateCommand<'a> {
    pub config: &'a mut AppState,
    pub directory: String,
    pub tags: Vec<String>,
    pub metadata: WidgetMetadataArgs,
}

impl<'a> CreateCommand<'a> {
    pub fn new(
        config: &'a mut AppState,
        directory: String,
        tags: Vec<String>,
        metadata: WidgetMetadataArgs,
    ) -> Self {
        Self {
            config,
            directory,
            tags,
            metadata,
        }
    }
}

impl Command for CreateCommand<'_> {
    fn execute(&mut self) -> String {
        if self.metadata.monitor.is_none() || self.metadata.layer.is_none() {
            return "monitor and layer are required".to_string();
        }

        let mut widget = Widget::new(
            &self.config.application,
            self.directory.to_owned(),
            self.tags.to_owned(),
        );
        let id = widget.id.to_owned();
        widget.update(self.metadata.to_owned());

        self.config.widgets.push(widget);
        id
    }
}
