use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};

use crate::widget::Widget;

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
        _ => Err("Invalid keyboard mode, possible values: [none, exclusive, on-demand]".to_string()),
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: CliCommands,
}

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct QueryArgs {
    #[clap(short, long)]
    pub id: Option<String>,

    #[clap(short, long)]
    pub directory: Option<String>,

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
        #[clap(short, long)]
        directory: String,

        #[clap(short, long)]
        tags: Vec<String>,

        #[clap(flatten)]
        metadata: WidgetMetadataArgs,
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

#[derive(Debug)]
pub struct AppState {
    pub application: gtk::Application,
    pub widgets: Vec<Widget>,
}
