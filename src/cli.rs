use clap::{Parser, Subcommand, Args};
use gtk_layer_shell::Layer;
use serde::{Serialize, Deserialize};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}


#[derive(Debug, Args, Serialize, Deserialize, Clone)]
pub struct ManageCommandArgs {
    #[clap(short, long)]
    pub id: Option<String>,

    #[clap(short, long)]
    pub directory: Option<String>,

    #[clap(short, long)]
    pub tags: Option<Vec<String>>,
}

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
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

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
pub struct WidgetDefaultSize {
    #[clap(long = "default-width")]
    pub width: Option<i32>,

    #[clap(long = "default-height")]
    pub height: Option<i32>,
}

#[derive(Debug, Args, Serialize, Deserialize)]
pub struct WidgetMetadataArgs {
    #[clap(short, long)]
    pub monitor: i32,

    #[clap(short, long)]
    pub layer: String,

    #[clap(short, long)]
    pub anchors: Vec<String>,

    #[clap(flatten)]
    pub margins: WidgetMargins,

    #[clap(flatten)]
    pub size: WidgetDefaultSize,
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
        tags: Option<Vec<String>>,

        #[clap(flatten)]
        metadata: WidgetMetadataArgs,
    },
    Delete {
        #[clap(flatten)]
        query: ManageCommandArgs,
    },
    Update {
        #[clap(flatten)]
        query: ManageCommandArgs,

        #[clap(flatten)]
        metadata: WidgetMetadataArgs,
    },
    Reload {
        #[clap(flatten)]
        query: ManageCommandArgs,
    },
    Show {
        #[clap(flatten)]
        query: ManageCommandArgs,
    },
    Hide {
        #[clap(flatten)]
        query: ManageCommandArgs,
    },
    List {
        #[clap(flatten)]
        query: ManageCommandArgs,
    },
    Version,
}
