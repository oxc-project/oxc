#[tokio::main]
async fn main() {
    #[expect(clippy::vec_init_then_push)]
    let tools: Vec<Box<dyn oxc_language_server::ToolBuilder>> = {
        let mut v: Vec<Box<dyn oxc_language_server::ToolBuilder>> = Vec::new();
        #[cfg(feature = "formatter")]
        v.push(Box::new(oxc_language_server::ServerFormatterBuilder));
        #[cfg(feature = "linter")]
        v.push(Box::new(oxc_language_server::ServerLinterBuilder));
        v
    };

    oxc_language_server::run_server(
        "oxc".to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
        tools,
    )
    .await;
}
