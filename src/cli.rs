use std::path::Path;

use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};

use crate::{app_state::AppState, utils::widget_filter};

fn parse_layer(s: &str) -> Result<String, String> {
    match s {
        "background" => Ok("background".to_string()),
        "bottom" => Ok("bottom".to_string()),
        "top" => Ok("top".to_string()),
        "overlay" => Ok("overlay".to_string()),
        _ => Err("Invalid layer, possible values: [background, bottom, top, overlay]".to_string()),
    }
}

fn parse_keyboard_mode(s: &str) -> Result<String, String> {
    match s {
        "none" => Ok("none".to_string()),
        "exclusive" => Ok("exclusive".to_string()),
        "on-demand" => Ok("on-demand".to_string()),
        _ => {
            Err("Invalid keyboard mode, possible values: [none, exclusive, on-demand]".to_string())
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: CliCommands,

    #[clap(short, long, default_value = "false")]
    pub json: bool,
}

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct QueryArgs {
    #[clap(short, long)]
    pub id: Option<String>,

    #[clap(short, long)]
    pub url: Option<String>,

    #[clap(short, long)]
    pub tags: Option<Vec<String>>,
}

#[derive(Parser, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WidgetMargins {
    #[clap(long = "margin-top")]
    pub top: Option<i32>,

    #[clap(long = "margin-right")]
    pub right: Option<i32>,

    #[clap(long = "margin-bottom")]
    pub bottom: Option<i32>,

    #[clap(long = "margin-left")]
    pub left: Option<i32>,
}

#[derive(Parser, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WidgetDefaultSize {
    #[clap(long = "default-width")]
    pub width: Option<i32>,

    #[clap(long = "default-height")]
    pub height: Option<i32>,
}

#[derive(Args, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WidgetMetadataArgs {
    #[clap(short, long)]
    pub monitor: Option<i32>,

    #[clap(short, long, value_parser = parse_layer)]
    pub layer: Option<String>,

    #[clap(short, long)]
    pub anchors: Option<Vec<String>>,

    #[clap(flatten)]
    pub margins: Option<WidgetMargins>,

    #[clap(flatten)]
    pub size: Option<WidgetDefaultSize>,

    #[clap(short, long)]
    pub click_through: Option<bool>,

    #[clap(short, long)]
    pub exclusive: Option<bool>,

    #[clap(short, long = "keyboard-mode", value_parser = parse_keyboard_mode)]
    pub keyboard_mode: Option<String>,
}

#[derive(Subcommand, Debug, Serialize, Deserialize)]
pub enum CliCommands {
    // list
    Init,
    Kill,
    Create {
        #[clap(flatten)]
        group: CreateUrlGroup,

        #[clap(short, long)]
        tags: Vec<String>,

        #[clap(flatten)]
        metadata: WidgetMetadataArgs,

        #[clap(short, long, default_value = "false")]
        show: bool,
    },
    Delete {
        #[clap(flatten)]
        query: QueryArgs,
    },
    Update {
        #[clap(flatten)]
        query: QueryArgs,

        #[clap(flatten)]
        metadata: WidgetMetadataArgs,
    },
    Reload {
        #[clap(flatten)]
        query: QueryArgs,
    },
    Show {
        #[clap(flatten)]
        query: QueryArgs,
    },
    Hide {
        #[clap(flatten)]
        query: QueryArgs,
    },
    List {
        #[clap(flatten)]
        query: QueryArgs,
    },
    Inspect {
        #[clap(flatten)]
        query: QueryArgs,
    },
    Version,
}

#[derive(Debug, Args, Clone, Serialize, Deserialize)]
#[group(required = true, multiple = false)]
pub struct CreateUrlGroup {
    #[clap(short, long)]
    directory: Option<String>,

    #[clap(short, long)]
    url: Option<String>,
}

pub trait CliCommand {
    fn mutate(&self, config: &mut AppState) -> String;
}

impl CliCommand for CliCommands {
    fn mutate(&self, config: &mut AppState) -> String {
        match self {
            CliCommands::List { query } => config
                .widgets
                .iter()
                .filter(|e| widget_filter(e, query))
                .map(|e| e.id.as_ref())
                .collect::<Vec<_>>()
                .join("\n"),
            CliCommands::Create {
                group,
                tags,
                metadata,
                show,
            } => {
                let i = group.directory.as_ref();
                let j = group.url.as_ref();

                let url = match i {
                    Some(d) => format!(
                        "http://localhost:8082/{}",
                        Path::new(d.as_str())
                            .join("index.html")
                            .to_str()
                            .unwrap()
                            .to_string()
                    ),
                    None => j.unwrap().to_owned(),
                };

                let ret = config.add_widget(url, tags.to_owned(), metadata.to_owned());

                if *show {
                    config.show_widget(&QueryArgs {
                        id: Some(ret.clone()),
                        url: None,
                        tags: None,
                    });
                }

                ret
            }
            CliCommands::Delete { query } => config.delete_widget(query),
            CliCommands::Update { query, metadata } => {
                config.update_widget(query, metadata.to_owned())
            }
            CliCommands::Hide { query } => config.hide_widget(query),
            CliCommands::Show { query } => config.show_widget(query),
            CliCommands::Kill => {
                config.kill_application();
                "killed".to_string()
            }
            CliCommands::Inspect { query } => config.inspect_widget(query),
            CliCommands::Reload { query } => config.reload_widget(query),
            // CARGO_PKG_VERSION
            CliCommands::Version => env!("CARGO_PKG_VERSION").to_string(),
            _ => "not implemented".to_string(),
        }
    }
}
