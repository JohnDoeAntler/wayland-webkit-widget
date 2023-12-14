use actix_web::{App, HttpServer, dev::ServerHandle};
use actix_web::rt;
use std::sync::mpsc;
use std::thread;

use crate::utils::get_widget_dir_path;

pub async fn create_web_server(tx: mpsc::Sender<ServerHandle>) -> std::io::Result<()> {
  let server = HttpServer
    ::new(|| App::new().service(actix_files::Files::new("/", get_widget_dir_path()).index_file("index.html")))
  .bind(("127.0.0.1", 8082))?
  .run();

  let _ = tx.send(server.handle());

  server.await
}

pub fn start_web_server() -> ServerHandle {
  let (tx, rx) = mpsc::channel();

  thread::spawn(move || {
    let server_future = create_web_server(tx);
    rt::System::new().block_on(server_future)
  });

  rx.recv().unwrap()
}

