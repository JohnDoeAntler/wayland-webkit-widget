use clap::{Parser, Subcommand};
use serde::{Serialize, Deserialize};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
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
    },
    Manage {
        #[clap(short, long)]
        id: Option<String>,

        #[clap(short, long)]
        directory: Option<String>,

        #[clap(short, long)]
        tags: Option<Vec<String>>,

        #[command(subcommand)]
        command: ManageCommands,
    },
    List,
}

#[derive(Subcommand, Debug, Serialize, Deserialize)]
pub enum ManageCommands {
    Delete,
    Update {},
    // no effects to the metadata
    Reload,
    Show,
    Hide,
}
