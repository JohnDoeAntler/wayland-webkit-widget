use gio::{prelude::*, ApplicationFlags};
use gtk::Application;
use std::{os::unix::net::UnixStream, thread};

use glib::clone;
use std::fmt::Formatter;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use gdk::cairo::RectangleInt;
use gdk::cairo::Region;
use gtk::{ApplicationWindow, gdk::Display};
use gio::{prelude::*};
// use gtk::prelude::{GtkWindowExt, WidgetExt, ContainerExt};
use gtk::prelude::*;
use gtk_layer_shell::{Edge, LayerShell};
use webkit2gtk::WebViewExt;
use webkit2gtk::WebView;
use gtk::glib;
use std::io::prelude::*;

use crate::{cli::Commands, utils::write_socket_message};

pub struct WidgetChannelMessage {
    pub stream: UnixStream,
    pub command: Commands,
}

pub fn start_widget_application(receiver: async_channel::Receiver<WidgetChannelMessage>) {
    thread::spawn(|| {
        gtk::init().unwrap();

        let app = Application::new(
            Some("org.gnome.webkit6-rs.example"),
            ApplicationFlags::FLAGS_NONE,
        );

        app.connect_activate(move |app| {
            println!("app activated");
            let receiver = receiver.clone();
            glib::spawn_future_local(async move {
                println!("widget thread started");
                let ret = receiver.recv();
                println!("preparing to receive message from daemon");
                match ret.await {
                    Ok(message) => {
                        let mut stream = message.stream;
                        let command = message.command;
                        write_socket_message(&mut stream, format!("ok, received {:?}", command));
                    }
                    Err(e) => {
                        println!("error: {:?}", e);
                    }
                }
            });
        });

        app.run_with_args::<&str>(&[]);
    });
}
