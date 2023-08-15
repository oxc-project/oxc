/// detect proxy from environment variable in following order:
/// ALL_PROXY | all_proxy | HTTPS_PROXY | https_proxy | HTTP_PROXY | http_proxy
fn detect_proxy() -> Option<ureq::Proxy> {
    macro_rules! try_env {
        ($($env:literal),+) => {
            $(
                if let Ok(env) = std::env::var($env) {
                    if let Ok(proxy) = ureq::Proxy::new(env) {
                        return Some(proxy);
                    }
                }
            )+
        };
    }

    try_env!("ALL_PROXY", "all_proxy", "HTTPS_PROXY", "https_proxy", "HTTP_PROXY", "http_proxy");
    None
}

/// build a agent with proxy automatically detected
pub fn agent() -> ureq::Agent {
    let builder = ureq::AgentBuilder::new();
    if let Some(proxy) = detect_proxy() {
        builder.proxy(proxy).build()
    } else {
        builder.build()
    }
}
