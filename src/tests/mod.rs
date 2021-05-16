pub mod fakes;
pub mod integration;
pub mod possible_tty_inputs;
pub mod tty_inputs;
pub mod utils;

use crate::cli::CliArgs;
use crate::client::start_client;
use crate::common::input::config::Config;
use crate::os_input_output::{ClientOsApi, ServerOsApi};
use crate::server::start_server;
use std::path::PathBuf;

pub fn start(
    client_os_input: Box<dyn ClientOsApi>,
    opts: CliArgs,
    server_os_input: Box<dyn ServerOsApi>,
    config: Config,
) {
    let server_thread = std::thread::Builder::new()
        .name("server_thread".into())
        .spawn(move || {
            start_server(server_os_input, PathBuf::from(""));
        })
        .unwrap();
    start_client(client_os_input, opts, config);
    let _ = server_thread.join();
}
