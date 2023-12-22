use crate::cli::{QueryArgs, WidgetMetadataArgs, WidgetDefaultSize, WidgetMargins};
use crate::constants::SOCKET_PATH;
use crate::utils::read_socket_response;
use crate::{cli::Commands, utils::write_socket_message};
use async_std::os::unix::net::UnixListener;
use gdk::Display;
use gio::{prelude::*, ApplicationFlags};
use gtk::glib;
use gtk::prelude::*;
use gtk::Application;
use gtk::ApplicationWindow;
use gtk_layer_shell::{Edge, Layer, LayerShell};
use std::path::Path;
use uuid::Uuid;
use webkit2gtk::{WebView, WebViewExt};

#[derive(Debug, PartialEq)]
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

    window.set_visual(Some(
        &WidgetExt::screen(&window).unwrap().rgba_visual().unwrap(),
    ));
    window.set_decorated(false);
    window.set_app_paintable(true);

    window
}

fn inject_javascript_to_webview(webview: &WebView, data: &Widget) {
    let template = r#"
        window.www = {};
        window.www.id = "{{id}}";
    "#
    .replace("{{id}}", data.id.as_str());

    webview.run_javascript(template.as_str(), gio::Cancellable::NONE, |e| {});
}

fn create_webview(url: String) -> WebView {
    let webview = WebView::new();

    webview.set_background_color(&gdk::RGBA::new(0.0, 0.0, 0.0, 0.0));
    webview.load_uri(url.as_str());

    webview
}

fn apply_layer_shell(window: &ApplicationWindow) {
    window.init_layer_shell();
}

fn update_monitor(window: &ApplicationWindow, monitor: &Option<i32>) {
    if let Some(monitor) = *monitor {
        let display = &Display::default().expect("failed to get display");
        let target_monitor = std::cmp::max(std::cmp::min(monitor, 0), display.n_monitors() - 1);
        window.set_monitor(display.monitor(target_monitor).unwrap().as_ref());
    }
}

fn update_layer(window: &ApplicationWindow, layer: &Option<String>) {
    if let Some(layer) = layer {
        window.set_layer(match layer.as_str() {
            "background" => Layer::Background,
            "bottom" => Layer::Bottom,
            "top" => Layer::Top,
            "overlay" => Layer::Overlay,
            _ => Layer::Top,
        });
    }
}

fn update_margins(window: &ApplicationWindow, margins: &WidgetMargins) {
    if let Some(top) = margins.left {
        window.set_layer_shell_margin(Edge::Top, top)
    }
    if let Some(right) = margins.right {
        window.set_layer_shell_margin(Edge::Right, right)
    }
    if let Some(bottom) = margins.bottom {
        window.set_layer_shell_margin(Edge::Bottom, bottom)
    }
    if let Some(left) = margins.left {
        window.set_layer_shell_margin(Edge::Left, left)
    }
}

fn update_anchors(window: &ApplicationWindow, anchors: &Vec<String>) {
    [Edge::Top, Edge::Right, Edge::Bottom, Edge::Left]
        .iter()
        .for_each(|e| window.set_anchor(*e, false));

    anchors.iter().for_each(|a| {
        window.set_anchor(
            match a.as_str() {
                "top" => Edge::Top,
                "right" => Edge::Right,
                "bottom" => Edge::Bottom,
                "left" => Edge::Left,
                _ => Edge::Top,
            },
            true,
        );
    });
}

fn update_size(window: &ApplicationWindow, size: &WidgetDefaultSize) {
    if let Some(width) = size.width {
        window.set_width_request(width);
    }

    if let Some(height) = size.height {
        window.set_height_request(height);
    }
}

impl Widget {
    fn show(&self) {
        self.window.show_all();
    }

    fn hide(&self) {
        self.window.hide();
    }

    fn close(&self) {
        self.window.close();
    }

    fn reload(&self) {
        self.webview.reload();
    }

    fn update(&self, metadata: WidgetMetadataArgs) {
        println!("metadata: {:?}", metadata);
        update_monitor(&self.window, &metadata.monitor);
        update_layer(&self.window, &metadata.layer);
        update_margins(&self.window, &metadata.margins);
        update_anchors(&self.window, &metadata.anchors);
        update_size(&self.window, &metadata.size);
        inject_javascript_to_webview(&self.webview, &self);
    }

    fn new(app: &Application, directory: String, tags: Vec<String>) -> Self {
        let window = create_window(app);
        let webview_url = format!(
            "http://localhost:8082/{}",
            Path::new(directory.as_str())
                .join("index.html")
                .to_str()
                .unwrap()
                .to_string()
        );

        let webview = create_webview(webview_url);
        window.add(&webview);
        apply_layer_shell(&window);

        Self {
            id: Uuid::new_v4().to_string(),
            tags,
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

fn query_widgets(w: &Vec<Widget>, query: QueryArgs) -> Vec<&Widget> {
    w.iter()
        .filter(|w| {
            (query.id.is_none() || query.id.as_ref().is_some_and(|e| w.id.contains(e)))
                && (query.directory.is_none()
                    || query
                        .directory
                        .as_ref()
                        .is_some_and(|e| w.directory.contains(e)))
                && (query.tags.is_none()
                    || query
                        .tags
                        .as_ref()
                        .is_some_and(|e| w.tags.iter().any(|t| e.contains(t))))
        })
        .collect()
}

async fn listen_unix_socket(app: Application) {
    if std::path::Path::new(SOCKET_PATH).exists() {
        std::fs::remove_file(SOCKET_PATH).expect("a daemon is already running");
    }

    let listener = UnixListener::bind(SOCKET_PATH).await.unwrap();
    let mut config = WidgetSet { widgets: vec![] };

    loop {
        let stream = listener.accept().await;

        if stream.is_err() {
            continue;
        }

        let (mut stream, _) = stream.unwrap();
        let command = read_socket_response(&mut stream).await;
        let command = serde_json::from_str::<Commands>(command.as_str()).unwrap();

        match command {
            Commands::Kill => {
                write_socket_message(&mut stream, "killed".to_string()).await;
                app.quit();
            }
            Commands::Create {
                directory,
                tags,
                metadata,
            } => {
                if metadata.monitor.is_none() || metadata.layer.is_none() {
                    write_socket_message(
                        &mut stream,
                        "monitor and layer are required".to_string(),
                    )
                    .await;
                    continue;
                }

                let id = {
                    let widget = Widget::new(&app, directory, tags);
                    widget.update(metadata);

                    let id = widget.id.to_owned();
                    config.widgets.push(widget);
                    id
                };
                write_socket_message(&mut stream, id).await;
            }
            Commands::Update { query, metadata } => {
                query_widgets(&config.widgets, query)
                    .iter()
                    .for_each(|w| w.update(metadata.to_owned()));
                write_socket_message(&mut stream, "ok".to_string()).await;
            }
            Commands::List { query } => {
                write_socket_message(
                    &mut stream,
                    format!("{:#?}", query_widgets(&config.widgets, query)),
                )
                .await;
            }
            Commands::Show { query } => {
                query_widgets(&config.widgets, query)
                    .iter()
                    .for_each(|w| w.show());
                write_socket_message(&mut stream, "ok".to_string()).await;
            }
            Commands::Hide { query } => {
                query_widgets(&config.widgets, query)
                    .iter()
                    .for_each(|w| w.hide());
                write_socket_message(&mut stream, "ok".to_string()).await;
            }
            Commands::Delete { query } => {
                let ids = query_widgets(&config.widgets, query)
                    .iter()
                    .map(|e| e.id.to_owned())
                    .collect::<Vec<String>>();

                config.widgets.iter().for_each(|w| {
                    if ids.contains(&w.id) {
                        w.close();
                    }
                });
                config.widgets.retain(|w| !ids.contains(&w.id));
                write_socket_message(&mut stream, "ok".to_string()).await;
            }
            Commands::Reload { query } => {
                query_widgets(&config.widgets, query)
                    .iter()
                    .for_each(|w| w.reload());
                write_socket_message(&mut stream, "ok".to_string()).await;
            }
            Commands::Version => {
                let display = &Display::default().expect("failed to get display");
                let num_of_monitor = display.n_monitors();
                let mut str = String::new();
                for i in 0..num_of_monitor {
                    let m = display.monitor(i).expect("failed to get monitor");
                    str.push_str(
                        format!(
                            "- monitor {}: [{}] {}\n",
                            i,
                            m.manufacturer()
                                .expect("failed to get model name of monitor"),
                            m.model().expect("failed to get model name of monitor")
                        )
                        .as_str(),
                    );
                }
                str.push_str("\nversion: 0.0.1");
                write_socket_message(&mut stream, str).await;
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
