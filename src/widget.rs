use gio::{prelude::*, ApplicationFlags};
use gtk::glib;
use gtk::prelude::*;
use gtk::Application;
use gtk::ApplicationWindow;
use std::{os::unix::net::UnixStream, thread};

use crate::{cli::Commands, utils::write_socket_message};

pub struct WidgetChannelMessage {
    pub stream: UnixStream,
    pub command: Commands,
}

pub fn start_widget_application(receiver: async_channel::Receiver<WidgetChannelMessage>) {
    thread::spawn(move || {
        gtk::init().unwrap();

        let app = Application::new(
            Some("org.gnome.webkit6-rs.example"),
            ApplicationFlags::FLAGS_NONE,
        );

        app.connect_activate(move |app| {
            println!("app activated");
            // dummy window for always awaking the glib main loop
            let _ = ApplicationWindow::new(app);
        });

        glib::spawn_future_local(async move {
            loop {
                match receiver.recv().await {
                    Ok(message) => {
                        let mut stream = message.stream;
                        let command = message.command;
                        write_socket_message(&mut stream, format!("ok, received {:?}", command));
                    }
                    Err(e) => {
                        println!("error: {:?}", e);
                    }
                }
            }
        });

        app.run_with_args::<&str>(&[]);
    });
}
