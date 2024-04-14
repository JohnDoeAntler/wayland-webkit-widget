use crate::app_state::AppState;
use crate::cli::{CliCommand, WidgetDefaultSize, WidgetMargins, WidgetMetadataArgs};
use crate::constants::SOCKET_PATH;
use crate::utils::read_socket_response;
use crate::{cli::CliCommands, utils::write_socket_message};
use async_std::channel;
use async_std::os::unix::net::UnixListener;
use gdk::cairo::{RectangleInt, Region};
use gdk::gio::{prelude::*, ApplicationFlags};
use gdk::Display;
use glib::{clone, SignalHandlerId};
use gtk::glib;
use gtk::prelude::*;
use gtk::Application;
use gtk::ApplicationWindow;
use gtk_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use serde::Serialize;
use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;
use webkit2gtk::{SettingsExt, UserContentManagerExt, WebInspectorExt, WebView, WebViewExt};

#[derive(Debug, Serialize, PartialEq, Default)]
pub struct WidgetMetadataAnchors {
    top: bool,
    right: bool,
    bottom: bool,
    left: bool,
}

#[derive(Debug, Serialize, PartialEq, Default)]
pub struct WidgetMetadataMargins {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct WidgetMetadataSize {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct WidgetMetadata {
    pub monitor: Option<i32>,
    #[serde(skip_serializing, skip_deserializing)]
    pub layer: Option<Layer>,
    pub anchors: Option<WidgetMetadataAnchors>,
    pub margins: Option<WidgetMetadataMargins>,
    pub size: Option<WidgetMetadataSize>,
    pub click_through: bool,
    pub exclusive: bool,
    pub keyboard_mode: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Widget {
    pub id: String,
    pub tags: Vec<String>,
    pub url: String,

    // gtk info
    #[serde(skip_serializing)]
    pub window: ApplicationWindow,
    #[serde(skip_serializing)]
    webview: WebView,
    pub metadata: WidgetMetadata,

    #[serde(skip_serializing)]
    signal_handler: Option<SignalHandlerId>,
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

fn inject_javascript_to_webview(widget: &Widget) -> SignalHandlerId {
    let template = r#"window.www = {{www}};"#
        .replace("{{www}}", serde_json::to_string(widget).unwrap().as_str());

    widget
        .webview
        .connect_load_changed(move |webview, load_event| {
            if load_event == webkit2gtk::LoadEvent::Finished {
                webview.run_javascript(
                    template.as_str(),
                    gdk::gio::Cancellable::NONE,
                    |_| {},
                );
            }
        })
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

fn apply_javascript_api(webview: &WebView, api: async_std::channel::Sender<String>) {
    let ucm = webview.user_content_manager().unwrap();

    let (tx, rx) = channel::unbounded();

    ucm.connect_script_message_received(Some("widget"), move |_, jsr| {
        // get arguments
        if let Some(args) = jsr.js_value() {
            let _ = tx.send_blocking(args.to_string());
        }
    });

    glib::spawn_future_local(clone!(@strong webview => async move {
        while let Ok(ret) = rx.recv().await {
            // parse it to CliCommands
            let _ = api.send_blocking(ret);
        }
    }));

    ucm.register_script_message_handler("widget");
}

fn update_monitor(window: &ApplicationWindow, monitor: i32) -> i32 {
    let display = &Display::default().expect("failed to get display");
    let target_monitor = std::cmp::min(std::cmp::max(monitor, 0), display.n_monitors() - 1);
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
    let mut ret = WidgetMetadataMargins::default();

    if let Some(top) = margins.top {
        window.set_layer_shell_margin(Edge::Top, top);
        ret.top = top;
    }

    if let Some(right) = margins.right {
        window.set_layer_shell_margin(Edge::Right, right);
        ret.right = right;
    }

    if let Some(bottom) = margins.bottom {
        window.set_layer_shell_margin(Edge::Bottom, bottom);
        ret.bottom = bottom;
    }

    if let Some(left) = margins.left {
        window.set_layer_shell_margin(Edge::Left, left);
        ret.left = left;
    }

    ret
}

fn update_anchors(window: &ApplicationWindow, anchors: &Vec<String>) -> WidgetMetadataAnchors {
    let mut ret = WidgetMetadataAnchors::default();

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

    pub fn inspect(&self) {
        let inspector = self.webview.inspector().unwrap();
        inspector.show();
    }

    pub fn update(&mut self, metadata: &WidgetMetadataArgs) {
        if let Some(monitor) = metadata.monitor {
            self.metadata.monitor = Some(update_monitor(&self.window, monitor));
        }
        if let Some(layer) = metadata.layer.as_ref() {
            self.metadata.layer = Some(update_layer(&self.window, layer.to_owned()));
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
        if let Some(keyboard_mode) = metadata.keyboard_mode.as_ref() {
            self.metadata.keyboard_mode =
                Some(update_keyboard_mode(&self.window, keyboard_mode.to_owned()));
        }
        if let Some(signal) = self.signal_handler.take() {
            self.webview.disconnect(signal);
        }
        self.signal_handler = Some(inject_javascript_to_webview(&self));
    }

    pub fn new(
        app: &Application,
        url: String,
        tags: Vec<String>,
        api: async_std::channel::Sender<String>,
    ) -> Self {
        let window = create_window(app);
        let webview = create_webview(url.to_owned());
        window.add(&webview);
        // init gtk layer shell
        apply_layer_shell(&window);
        // inject ipc
        apply_javascript_api(&webview, api);

        // enable webkit inspector
        let settings = WebViewExt::settings(&webview).unwrap();
        settings.set_enable_developer_extras(true);

        // create widget
        let widget = Self {
            id: Uuid::new_v4().to_string(),
            tags,
            url,
            window,
            webview,
            signal_handler: None,
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
        };

        widget
    }
}

async fn listen_unix_socket(state: Rc<RefCell<AppState>>) {
    if std::path::Path::new(SOCKET_PATH).exists() {
        std::fs::remove_file(SOCKET_PATH).expect("a daemon is already running");
    }

    let listener = UnixListener::bind(SOCKET_PATH).await.unwrap();

    while let Ok((mut stream, _)) = listener.accept().await {
        let command = read_socket_response(&mut stream).await;
        let command = serde_json::from_str::<CliCommands>(command.as_str()).unwrap();

        let mut app_state = RefCell::borrow_mut(&state);
        write_socket_message(&mut stream, command.mutate(&mut app_state)).await;
    }
}

async fn listen_webkit_messages(
    state: Rc<RefCell<AppState>>,
    rx: async_std::channel::Receiver<String>,
) {
    while let Ok(message) = rx.recv().await {
        let command = serde_json::from_str::<CliCommands>(message.as_str()).unwrap();
        command.mutate(&mut RefCell::borrow_mut(&state));
    }
}

pub fn start_widget_application() {
    gtk::init().unwrap();

    let app = Application::new(
        Some("com.johndoeantler.wayland-webkit-widget"),
        ApplicationFlags::FLAGS_NONE,
    );

    app.connect_activate(move |application| {
        // dummy window for always awaking the glib main loop
        let _ = ApplicationWindow::new(application);

        // ipc channel
        let (tx, rx) = channel::unbounded();
        let shared_state = Rc::new(RefCell::new(AppState::new(application.to_owned(), tx)));
        let state_for_widget = shared_state.clone();
        let state_for_ipc = shared_state.clone();

        // listen the socket
        glib::spawn_future_local(async move {
            listen_unix_socket(state_for_widget).await;
        });

        // handle any messages sent from javascript webkit api
        glib::spawn_future_local(async move {
            listen_webkit_messages(state_for_ipc, rx).await;
        });
    });

    app.run_with_args::<&str>(&[]);
}
