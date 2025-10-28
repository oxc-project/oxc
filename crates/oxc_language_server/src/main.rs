#[tokio::main]
async fn main() {
    oxc_language_server::run_server().await;
}
