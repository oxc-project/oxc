#[tokio::main]
async fn main() {
    oxc_language_server::run_server(vec![
        Box::new(oxc_language_server::ServerFormatterBuilder),
        Box::new(oxc_language_server::ServerLinterBuilder),
    ])
    .await;
}
