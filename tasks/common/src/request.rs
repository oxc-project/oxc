use std::time::Duration;

use ureq::{Agent, Proxy};

/// detect proxy from environment variable in following order:
/// ALL_PROXY | all_proxy | HTTPS_PROXY | https_proxy | HTTP_PROXY | http_proxy
fn detect_proxy() -> Option<Proxy> {
    for env in ["HTTPS_PROXY", "https_proxy", "HTTP_PROXY", "http_proxy", "ALL_PROXY", "all_proxy"]
    {
        if let Ok(env) = std::env::var(env) {
            if let Ok(proxy) = Proxy::new(&env) {
                return Some(proxy);
            }
        }
    }
    None
}

/// build a agent with proxy automatically detected
pub fn agent() -> Agent {
    let config = Agent::config_builder()
        .proxy(detect_proxy())
        .timeout_global(Some(Duration::from_secs(5)))
        .build();
    Agent::new_with_config(config)
}
