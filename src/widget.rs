use crate::constants::SOCKET_PATH;
use crate::utils::read_socket_response;
use crate::{cli::Commands, cli::ManageCommands, utils::write_socket_message};
use async_std::os::unix::net::UnixListener;
use gio::{prelude::*, ApplicationFlags};
use gtk::glib;
use gtk::Application;
use gtk::ApplicationWindow;
use gtk::prelude::*;
use uuid::Uuid;
use webkit2gtk::{WebView, WebViewExt};

#[derive(Debug)]
struct Widget {
    id: String,
    tags: Vec<String>,
    directory: String,

    // gtk info
    window: ApplicationWindow,
    webview: WebView,
}

fn create_window(app: &Application) -> ApplicationWindow {
    let window = ApplicationWindow::new(app);

    window.set_visual(Some(&WidgetExt::screen(&window).unwrap().rgba_visual().unwrap()));
    window.set_decorated(false);
    window.set_app_paintable(true);

    window
}

fn create_webview(url: String) -> WebView {
    let webview = WebView::new();

    webview.set_background_color(&gdk::RGBA::new(0.0, 0.0, 0.0, 0.0));
    webview.load_uri(url.as_str());
    webview.run_javascript_future("window.monitor = 1");

    webview
}

impl Widget {
    fn show(&self) {
        self.window.show();
    }

    fn hide(&self) {
        self.window.hide();
    }

    fn reload(&self) {
        self.webview.reload();
    }

    fn new(app: &Application, directory: String, tags: Option<Vec<String>>) -> Self {
        let window = create_window(app);
        let webview = create_webview("https://www.google.com".to_string());
        window.add(&webview);

        Self {
            id: Uuid::new_v4().to_string(),
            tags: tags.unwrap_or(vec![]),
            directory,
            window,
            webview,
        }
    }
}

#[derive(Debug)]
struct WidgetSet {
    widgets: Vec<Widget>,
}

async fn listen_unix_socket(app: Application) {
    if std::path::Path::new(SOCKET_PATH).exists() {
        std::fs::remove_file(SOCKET_PATH).expect("a daemon is already running");
    }

    let listener = UnixListener::bind(SOCKET_PATH).await.unwrap();
    let mut set = WidgetSet { widgets: vec![] };

    loop {
        let stream = listener.accept().await;

        if stream.is_err() {
            continue;
        }

        let (mut stream, _) = stream.unwrap();
        let command = read_socket_response(&mut stream).await;
        let command = serde_json::from_str::<Commands>(command.as_str()).unwrap();

        // we only listen kill command here, other commands should be passed to the gtk thread
        match command {
            Commands::Kill => {
                write_socket_message(&mut stream, "killed".to_string()).await;
                app.quit();
            }
            Commands::Create { directory, tags } => {
                let w = Widget::new(&app, directory, tags);
                write_socket_message(&mut stream, w.id.to_owned()).await;
                set.widgets.push(w);
            }
            Commands::List => {
                write_socket_message(&mut stream, format!("{:?}", set.widgets)).await;
            }
            Commands::Manage { id, directory, tags, command } => {
                // filter widgets by user query
                let widget_selected = set.widgets.iter().filter(|w| {
                    (id.is_none() || id.as_ref().is_some_and(|e| w.id.contains(e)))
                    && (directory.is_none() || directory.as_ref().is_some_and(|e| w.directory.contains(e)))
                    && (tags.is_none() || tags.as_ref().is_some_and(|e| w.tags.iter().any(|t| e.contains(t))))
                }).collect::<Vec<_>>();

                match command {
                    ManageCommands::Show => {
                        widget_selected.iter().for_each(|w| w.show());
                    }
                    ManageCommands::Hide => {
                        widget_selected.iter().for_each(|w| w.hide());
                    }
                    ManageCommands::Delete => {
                        widget_selected.iter().for_each(|w| w.hide());
                        widget_selected.iter().for_each(|e| {
                            let k = set.widgets.iter_mut();
                            k.retain(|w| w.id != e.id)
                        });
                    }
                    _ => {
                        write_socket_message(&mut stream, format!("not implemented, selected: {:?}", widget_selected)).await;
                    }
                }
            }
            _ => {
                write_socket_message(&mut stream, "not implemented".to_string()).await;
            }
        }
    }
}

pub fn start_widget_application() {
    gtk::init().unwrap();

    let app = Application::new(
        Some("org.gnome.webkit6-rs.example"),
        ApplicationFlags::FLAGS_NONE,
    );

    app.connect_activate(move |app| {
        // dummy window for always awaking the glib main loop
        let _ = ApplicationWindow::new(app);
        let app = app.to_owned();

        glib::spawn_future_local(async move {
            listen_unix_socket(app).await;
        });
    });

    app.run_with_args::<&str>(&[]);
}
