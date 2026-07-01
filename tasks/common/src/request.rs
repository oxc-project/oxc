use std::time::Duration;

use ureq::{Agent, Proxy};

/// detect proxy from environment variable in following order:
/// HTTPS_PROXY | https_proxy | HTTP_PROXY | http_proxy | ALL_PROXY | all_proxy
fn detect_proxy() -> Option<Proxy> {
    for env in ["HTTPS_PROXY", "https_proxy", "HTTP_PROXY", "http_proxy", "ALL_PROXY", "all_proxy"]
    {
        if let Ok(env) = std::env::var(env)
            && let Ok(proxy) = Proxy::new(&env)
        {
            return Some(proxy);
        }
    }
    None
}

/// build a agent with proxy automatically detected
pub fn agent() -> Agent {
    let config = Agent::config_builder()
        .proxy(detect_proxy())
        .timeout_global(Some(Duration::from_secs(10)))
        .build();
    Agent::new_with_config(config)
}

/// Build an agent for talking to servers on localhost.
///
/// No proxy (proxy env vars must not reroute loopback traffic) and a generous
/// timeout — the server bounds its own per-request work, but on a loaded
/// machine requests can queue well past `agent()`'s 10 second global timeout.
pub fn local_agent() -> Agent {
    let config = Agent::config_builder().timeout_global(Some(Duration::from_mins(1))).build();
    Agent::new_with_config(config)
}
