mod app;
mod cli;
mod constants;
mod utils;
mod http_server;
mod widget;

use std::os::unix::net::UnixStream;

use app::Daemon;
use clap::Parser;
use cli::{Cli, Commands};
use constants::SOCKET_PATH;
use serde_json;
use utils::{read_socket_response, write_socket_message};

fn main() {
    // parse from cli
    let cli = Cli::parse();

    // if init, start daemon
    match &cli.command {
        Commands::Init => {
            let daemon = Daemon::new();
            daemon.listen();
            return;
        }
        _ => (),
    }

    // else parse the command and send it to the daemon
    let mut stream = UnixStream::connect(SOCKET_PATH).expect("daemon is not running");
    let k = serde_json::to_string(&cli.command).unwrap().to_string();

    write_socket_message(&mut stream, k);
    println!("{}", read_socket_response(&mut stream));
}
