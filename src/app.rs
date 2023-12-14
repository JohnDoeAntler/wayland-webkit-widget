use crate::http_server::start_web_server;
use crate::widget::{start_widget_application, WidgetChannelMessage};
use crate::{constants::SOCKET_PATH, utils::write_socket_message, Commands};

use actix_web::dev::ServerHandle;
use actix_web::rt;

use crate::utils::read_socket_response;
use async_channel;
use daemonize::Daemonize;
use std::{fs::File, os::unix::net::UnixListener};

pub struct Daemon {
    server_handle: ServerHandle,
    widget_channel_sender: async_channel::Sender<WidgetChannelMessage>,
}

impl Daemon {
    pub fn new() -> Self {
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

        // init widget application
        let (widget_channel_sender, receiver) = async_channel::unbounded();
        start_widget_application(receiver.clone());

        // init web server
        let server_handle = start_web_server();

        // return
        Self {
            server_handle,
            widget_channel_sender,
        }
    }

    pub fn cleanup(&self) {
        rt::System::new().block_on(self.server_handle.stop(true));
        println!("stopped web server");
        println!("process exited");
    }

    pub fn listen(&self) {
        if std::path::Path::new(SOCKET_PATH).exists() {
            std::fs::remove_file(SOCKET_PATH).expect("a daemon is already running");
        }

        let listener = UnixListener::bind(SOCKET_PATH).expect("failed to bind unix socket");

        for stream in listener.incoming() {
            if stream.is_err() {
                continue;
            }

            let mut stream = stream.unwrap();
            let command = read_socket_response(&mut stream);
            let command = serde_json::from_str::<Commands>(command.as_str()).unwrap();

            // we only listen kill command here, other commands should be passed to the gtk thread
            match command {
                Commands::Kill => {
                    write_socket_message(&mut stream, "killed".to_string());
                    break;
                }
                _ => {
                    println!("received message: {:?}", command);
                    let sender = &self.widget_channel_sender;
                    sender.send_blocking(WidgetChannelMessage { stream, command }).unwrap();
                    println!("sent message to widget thread");
                }
            }
        }

        self.cleanup();
    }
}
