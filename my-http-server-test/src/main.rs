#![allow(unused)]
use std::sync::Arc;

use rust_extensions::AppStates;

mod http;

mod tests;
#[tokio::main]
async fn main() {
    let app_states = AppStates::create_initialized();
    let app_states = Arc::new(app_states);
    crate::http::start_up::setup_server(app_states.clone());

    app_states.wait_until_shutdown().await;
}
