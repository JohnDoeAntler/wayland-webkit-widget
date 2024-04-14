use crate::{
    cli::{QueryArgs, WidgetMetadataArgs},
    utils::widget_filter,
    widget::Widget,
};

use gdk::prelude::ApplicationExt;

#[derive(Debug)]
pub struct AppState {
    pub application: gtk::Application,
    pub widgets: Vec<Widget>,
    pub api: async_std::channel::Sender<String>,
}

impl AppState {
    pub fn new(application: gtk::Application, api: async_std::channel::Sender<String>) -> Self {
        Self {
            application,
            widgets: vec![],
            api,
        }
    }

    pub fn add_widget(
        &mut self,
        url: String,
        tags: Vec<String>,
        metadata: WidgetMetadataArgs,
    ) -> String {
        let mut widget = Widget::new(&self.application, url, tags, self.api.clone());
        let id = widget.id.to_owned();

        // update widget metadata
        widget.update(&metadata);

        // add widget to config
        self.widgets.push(widget);

        id
    }

    pub fn update_widget(&mut self, query: &QueryArgs, metadata: WidgetMetadataArgs) -> String {
        self.widgets
            .iter_mut()
            .filter(|w| widget_filter(w, &query))
            .map(|e| {
                e.update(&metadata);
                e.id.as_ref()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn delete_widget(&mut self, query: &QueryArgs) -> String {
        let mut ret = String::new();

        self.widgets.retain(|w| {
            if widget_filter(w, query) {
                w.close();
                ret.push_str(&w.id);
                false
            } else {
                true
            }
        });

        ret.to_string()
    }

    pub fn hide_widget(&self, query: &QueryArgs) -> String {
        self.widgets
            .iter()
            .filter(|w| widget_filter(w, &query))
            .map(|e| {
                e.hide();
                e.id.as_ref()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn show_widget(&self, query: &QueryArgs) -> String {
        self.widgets
            .iter()
            .filter(|w| widget_filter(w, &query))
            .map(|e| {
                e.show();
                e.id.as_ref()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn inspect_widget(&self, query: &QueryArgs) -> String {
        self.widgets
            .iter()
            .filter(|w| widget_filter(w, &query))
            .map(|e| {
                e.inspect();
                e.id.as_ref()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn kill_application(&self) {
        self.application.quit();
    }

    pub fn reload_widget(&self, query: &QueryArgs) -> String {
        self.widgets
            .iter()
            .filter(|w| widget_filter(w, &query))
            .map(|e| {
                e.reload();
                e.id.as_ref()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
