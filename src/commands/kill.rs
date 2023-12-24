use gdk::prelude::ApplicationExt;

use super::Command;
use crate::cli::AppState;

pub struct KillCommand<'a> {
    pub config: &'a mut AppState,
}

impl<'a> KillCommand<'a> {
    pub fn new(config: &'a mut AppState) -> Self {
        Self { config }
    }
}

impl Command for KillCommand<'_> {
    fn execute(&mut self) -> String {
        self.config.application.quit();
        "killed".to_string()
    }
}
