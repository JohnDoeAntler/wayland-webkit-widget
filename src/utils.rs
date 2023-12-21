use async_std::io::{prelude::BufReadExt, BufReader, WriteExt};
use async_std::os::unix::net::UnixStream;
use std::env;
use std::path::{Path, PathBuf};

pub async fn write_socket_message(stream: &mut UnixStream, message: String) {
    let n = message.split("\n").count();
    let mut output = String::new();

    output.push_str(&n.to_string());
    output.push_str("\n");
    output.push_str(message.as_str());
    output.push_str("\n");

    stream
        .write_all(output.as_bytes())
        .await
        .expect("failed to write");
}

pub async fn read_socket_response(stream: &mut UnixStream) -> String {
    let mut input = String::new();
    let mut reader = BufReader::new(stream);
    reader.read_line(&mut input).await.expect("failed to read");

    let n = input.trim().parse::<u32>().unwrap();
    input.clear();

    for _ in 0..n {
        let mut line = String::new();
        reader.read_line(&mut line).await.expect("failed to read");
        input.push_str(&line);
    }

    // remove the extra newline
    input.pop();

    input
}

pub fn get_widget_dir_path() -> PathBuf {
    let default_config_path = format!("{}/.config", std::env::var("HOME").unwrap());
    let xdg_config_path = env::var("XDG_CONFIG_HOME").unwrap_or(default_config_path);

    Path::new(xdg_config_path.as_str()).join("www")
}
