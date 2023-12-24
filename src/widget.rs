use crate::cli::{AppState, WidgetDefaultSize, WidgetMargins, WidgetMetadataArgs};
use crate::commands::handler::CommandHandler;
use crate::constants::SOCKET_PATH;
use crate::utils::read_socket_response;
use crate::{cli::CliCommands, utils::write_socket_message};
use async_std::os::unix::net::UnixListener;
use gdk::cairo::{RectangleInt, Region};
use gdk::Display;
use gio::{prelude::*, ApplicationFlags};
use gtk::glib;
use gtk::prelude::*;
use gtk::Application;
use gtk::ApplicationWindow;
use gtk_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::path::Path;
use uuid::Uuid;
use webkit2gtk::{SettingsExt, WebView, WebViewExt};

#[derive(Debug, PartialEq)]
pub struct WidgetMetadataAnchors {
    top: bool,
    right: bool,
    bottom: bool,
    left: bool,
}

#[derive(Debug, PartialEq)]
pub struct WidgetMetadataMargins {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

#[derive(Debug, PartialEq)]
pub struct WidgetMetadataSize {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, PartialEq)]
pub struct WidgetMetadata {
    pub monitor: Option<i32>,
    pub layer: Option<Layer>,
    pub anchors: Option<WidgetMetadataAnchors>,
    pub margins: Option<WidgetMetadataMargins>,
    pub size: Option<WidgetMetadataSize>,
    pub click_through: bool,
    pub exclusive: bool,
    pub keyboard_mode: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct Widget {
    pub id: String,
    pub tags: Vec<String>,
    pub directory: String,

    // gtk info
    pub window: ApplicationWindow,
    pub webview: WebView,
    pub metadata: WidgetMetadata,
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

    webview.run_javascript(template.as_str(), gio::Cancellable::NONE, |e| {
        if let Ok(r) = e {
            println!("javascript injected: {:?}", r.to_value());
        } else if let Err(e) = e {
            println!("error: {:?}", e);
        }
    });
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

fn update_monitor(window: &ApplicationWindow, monitor: i32) -> i32 {
    let display = &Display::default().expect("failed to get display");
    let target_monitor = std::cmp::max(std::cmp::min(monitor, 0), display.n_monitors() - 1);
    window.set_monitor(display.monitor(target_monitor).unwrap().as_ref());
    target_monitor
}

fn update_layer(window: &ApplicationWindow, layer: String) -> Layer {
    let layer = match layer.as_str() {
        "background" => Layer::Background,
        "bottom" => Layer::Bottom,
        "top" => Layer::Top,
        "overlay" => Layer::Overlay,
        _ => Layer::Background,
    };

    window.set_layer(layer);

    layer
}

fn update_margins(window: &ApplicationWindow, margins: &WidgetMargins) -> WidgetMetadataMargins {
    WidgetMetadataMargins {
        top: if let Some(top) = margins.top {
            window.set_layer_shell_margin(Edge::Top, top);
            top
        } else {
            0
        },
        right: if let Some(right) = margins.right {
            window.set_layer_shell_margin(Edge::Right, right);
            right
        } else {
            0
        },
        bottom: if let Some(bottom) = margins.bottom {
            window.set_layer_shell_margin(Edge::Bottom, bottom);
            bottom
        } else {
            0
        },
        left: if let Some(left) = margins.left {
            window.set_layer_shell_margin(Edge::Left, left);
            left
        } else {
            0
        },
    }
}

fn update_anchors(window: &ApplicationWindow, anchors: &Vec<String>) -> WidgetMetadataAnchors {
    let mut ret = WidgetMetadataAnchors {
        top: false,
        right: false,
        bottom: false,
        left: false,
    };

    if anchors.is_empty() {
        return ret;
    }

    [Edge::Top, Edge::Right, Edge::Bottom, Edge::Left]
        .iter()
        .for_each(|e| window.set_anchor(*e, false));

    anchors.iter().for_each(|a| {
        window.set_anchor(
            match a.as_str() {
                "top" => {
                    ret.top = true;
                    Edge::Top
                }
                "right" => {
                    ret.right = true;
                    Edge::Right
                }
                "bottom" => {
                    ret.bottom = true;
                    Edge::Bottom
                }
                "left" => {
                    ret.left = true;
                    Edge::Left
                }
                _ => {
                    ret.top = true;
                    Edge::Top
                }
            },
            true,
        );
    });

    ret
}

fn update_size(window: &ApplicationWindow, size: &WidgetDefaultSize) -> WidgetMetadataSize {
    let mut ret = WidgetMetadataSize {
        width: -1,
        height: -1,
    };

    if let Some(width) = size.width {
        window.set_width_request(width);
        ret.width = width;
    }

    if let Some(height) = size.height {
        window.set_height_request(height);
        ret.height = height;
    }

    ret
}

fn update_click_through(window: &ApplicationWindow, click_through: bool) -> bool {
    if click_through {
        let rectangle_int = RectangleInt::new(0, 0, 0, 0);
        let rectangle = Region::create_rectangle(&rectangle_int);
        window.input_shape_combine_region(Some(&rectangle));
        true
    } else {
        window.input_shape_combine_region(None);
        false
    }
}

fn update_exclusive(window: &ApplicationWindow, exclusive: bool) -> bool {
    if exclusive {
        window.auto_exclusive_zone_enable();
        true
    } else {
        window.set_exclusive_zone(0);
        false
    }
}

fn update_keyboard_mode(window: &ApplicationWindow, keyboard_mode: String) -> String {
    // window.set_keyboard_interactivity(keyboard_interactivity);
    match keyboard_mode.as_str() {
        "none" => window.set_keyboard_mode(KeyboardMode::None),
        "on-demand" => window.set_keyboard_mode(KeyboardMode::OnDemand),
        "exclusive" => window.set_keyboard_mode(KeyboardMode::Exclusive),
        _ => window.set_keyboard_mode(KeyboardMode::None),
    }
    keyboard_mode
}

impl Widget {
    pub fn show(&self) {
        self.window.show_all();
    }

    pub fn hide(&self) {
        self.window.hide();
    }

    pub fn close(&self) {
        self.window.close();
    }

    pub fn reload(&self) {
        self.webview.reload();
    }

    pub fn update(&mut self, metadata: WidgetMetadataArgs) {
        println!("metadata: {:?}", metadata);
        if let Some(monitor) = metadata.monitor {
            self.metadata.monitor = Some(update_monitor(&self.window, monitor));
        }
        if let Some(layer) = metadata.layer {
            self.metadata.layer = Some(update_layer(&self.window, layer));
        }
        if let Some(margins) = &metadata.margins {
            self.metadata.margins = Some(update_margins(&self.window, margins));
        }
        if let Some(anchors) = &metadata.anchors {
            self.metadata.anchors = Some(update_anchors(&self.window, anchors));
        }
        if let Some(size) = &metadata.size {
            self.metadata.size = Some(update_size(&self.window, size));
        }
        if let Some(click_through) = metadata.click_through {
            self.metadata.click_through = update_click_through(&self.window, click_through);
        }
        if let Some(exclusive) = metadata.exclusive {
            self.metadata.click_through = update_exclusive(&self.window, exclusive);
        }
        if let Some(keyboard_mode) = metadata.keyboard_mode {
            self.metadata.keyboard_mode = Some(update_keyboard_mode(&self.window, keyboard_mode));
        }

        inject_javascript_to_webview(&self.webview, &self);
    }

    pub fn new(app: &Application, directory: String, tags: Vec<String>) -> Self {
        let window = create_window(app);
        let webview = create_webview(format!(
            "http://localhost:8082/{}",
            Path::new(directory.as_str())
                .join("index.html")
                .to_str()
                .unwrap()
                .to_string()
        ));
        window.add(&webview);
        apply_layer_shell(&window);

        // enable webkit inspector
        let settings = WebViewExt::settings(&webview).unwrap();
        settings.set_enable_developer_extras(true);

        Self {
            id: Uuid::new_v4().to_string(),
            tags,
            directory,
            window,
            webview,
            metadata: WidgetMetadata {
                monitor: None,
                layer: None,
                margins: None,
                anchors: None,
                size: None,
                click_through: false,
                exclusive: false,
                keyboard_mode: None,
            },
        }
    }
}

async fn listen_unix_socket(application: Application) {
    if std::path::Path::new(SOCKET_PATH).exists() {
        std::fs::remove_file(SOCKET_PATH).expect("a daemon is already running");
    }

    let listener = UnixListener::bind(SOCKET_PATH).await.unwrap();

    let mut config = AppState {
        application,
        widgets: vec![],
    };
    let mut handler = CommandHandler::new();

    loop {
        let stream = listener.accept().await;

        if stream.is_err() {
            continue;
        }

        let (mut stream, _) = stream.unwrap();
        let command = read_socket_response(&mut stream).await;
        let command = serde_json::from_str::<CliCommands>(command.as_str()).unwrap();

        write_socket_message(&mut stream, handler.handle(command, &mut config)).await;
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
