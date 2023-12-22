use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
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

#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
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

#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
pub struct WidgetDefaultSize {
    #[clap(long = "default-width")]
    pub width: Option<i32>,

    #[clap(long = "default-height")]
    pub height: Option<i32>,
}

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct WidgetMetadataArgs {
    #[clap(short, long)]
    pub monitor: Option<i32>,

    #[clap(short, long)]
    pub layer: Option<String>,

    #[clap(short, long)]
    pub anchors: Vec<String>,

    #[clap(flatten)]
    pub margins: WidgetMargins,

    #[clap(flatten)]
    pub size: WidgetDefaultSize,

    #[clap(short, long, default_value = "false")]
    pub click_through: bool,

    #[clap(short, long, default_value = "false")]
    pub exclusive: bool,

    #[clap(short, long = "keyboard-interactivity", default_value = "false")]
    pub keyboard_interactivity: bool,
}

#[derive(Subcommand, Debug, Serialize, Deserialize)]
pub enum Commands {
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
    Version,
}
