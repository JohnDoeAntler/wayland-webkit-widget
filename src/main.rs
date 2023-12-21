mod app;
mod cli;
mod constants;
mod http_server;
mod utils;
mod widget;
use std::fs::File;

use clap::Parser;
use cli::{Cli, Commands};
use constants::SOCKET_PATH;
use daemonize::Daemonize;
use http_server::start_web_server;
use serde_json;
use async_std::os::unix::net::UnixStream;
use utils::{read_socket_response, write_socket_message};
use actix_web::rt;
use widget::start_widget_application;

pub fn daemonize_application() {
    // daemonize
    let stdout = File::create("/tmp/daemon.out").unwrap();
    let stderr = File::create("/tmp/daemon.err").unwrap();

    let daemonize = Daemonize::new()
        .pid_file("/tmp/wayland-webkit-widget.pid") // Every method except `new` and `start`
        .working_directory("/tmp") // for default behaviour.
        .stdout(stdout) // Redirect stdout to `/tmp/daemon.out`.
        .stderr(stderr) // Redirect stderr to `/tmp/daemon.err`.
        .privileged_action(|| "Executed before drop privileges");

    match daemonize.start() {
        Ok(_) => (),
        Err(e) => println!("Error, {}", e),
    };
}

fn main() {
    // parse from cli
    let cli = Cli::parse();

    // if init, start daemon
    match &cli.command {
        Commands::Init => {
            // 1. daemonize
            daemonize_application();

            // 2. start http server
            let server_handle = start_web_server();

            // 3. start gtk application, this will block the main thread
            start_widget_application();

            // 4. kill http server handle after gtk application is closed
            rt::System::new().block_on(server_handle.stop(true));
        }
        _ => {
            // run async statements with actix runtime
            rt::System::new().block_on(async {
                // else parse the command and send it to the daemon
                let mut stream = UnixStream::connect(SOCKET_PATH).await.expect("daemon is not running");
                let k = serde_json::to_string(&cli.command).unwrap().to_string();

                write_socket_message(&mut stream, k).await;
                println!("{}", read_socket_response(&mut stream).await);
            });
        },
    }
}
