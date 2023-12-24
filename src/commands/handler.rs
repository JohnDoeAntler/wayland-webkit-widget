use crate::cli::{AppState, CliCommands};

use super::Command;

pub struct CommandHandler {}

impl CommandHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle(&mut self, command: CliCommands, config: &mut AppState) -> String {
        let command: Option<Box<dyn Command>> = match command {
            CliCommands::List { query } => {
                Some(Box::new(super::list::ListCommand::new(config, query)))
            }
            CliCommands::Kill => Some(Box::new(super::kill::KillCommand::new(config))),
            CliCommands::Create {
                directory,
                tags,
                metadata,
            } => Some(Box::new(super::create::CreateCommand::new(
                config, directory, tags, metadata,
            ))),
            CliCommands::Update { query, metadata } => Some(Box::new(
                super::update::UpdateCommand::new(config, query, metadata),
            )),
            CliCommands::Show { query } => {
                Some(Box::new(super::show::ShowCommand::new(config, query)))
            }
            CliCommands::Hide { query } => {
                Some(Box::new(super::hide::HideCommand::new(config, query)))
            }
            CliCommands::Delete { query } => {
                Some(Box::new(super::delete::DeleteCommand::new(config, query)))
            }
            CliCommands::Reload { query } => {
                Some(Box::new(super::reload::ReloadCommand::new(config, query)))
            }
            CliCommands::Inspect { query } => {
                Some(Box::new(super::inspect::InspectCommand::new(config, query)))
            }
            _ => None,
        };

        if let Some(mut command) = command {
            command.execute()
        } else {
            "not implemented".to_string()
        }
    }
}
