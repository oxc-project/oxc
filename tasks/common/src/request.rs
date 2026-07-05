use std::{fs, path::PathBuf, time::Duration};

use ureq::{
    Agent, Proxy,
    tls::{PemItem, RootCerts, TlsConfig, TlsProvider, parse_pem},
};

// Warp's TLS proxy requires the macOS platform verifier, while using native TLS on Linux pulls
// OpenSSL into cross-target builds.
#[cfg(target_os = "macos")]
const TLS_PROVIDER: TlsProvider = TlsProvider::NativeTls;
#[cfg(not(target_os = "macos"))]
const TLS_PROVIDER: TlsProvider = TlsProvider::Rustls;

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
    let mut config =
        Agent::config_builder().proxy(detect_proxy()).timeout_global(Some(Duration::from_secs(10)));
    if let Some(tls_config) = tls_config_from_env() {
        config = config.tls_config(tls_config);
    }
    let config = config.build();
    Agent::new_with_config(config)
}

/// Use the custom root certificate bundle specified by `SSL_CERT_FILE`, when set.
///
/// This is useful for environments where HTTPS traffic is inspected by a proxy with a private
/// certificate authority. The file must contain one or more PEM-encoded certificates.
fn tls_config_from_env() -> Option<TlsConfig> {
    let path = std::env::var_os("SSL_CERT_FILE")
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())?;
    let pem = fs::read(&path)
        .unwrap_or_else(|error| panic!("failed to read SSL_CERT_FILE {}: {error}", path.display()));
    let certificates = parse_pem(&pem)
        .map(|item| {
            item.unwrap_or_else(|error| {
                panic!("failed to parse SSL_CERT_FILE {}: {error}", path.display())
            })
        })
        .filter_map(|item| match item {
            PemItem::Certificate(certificate) => Some(certificate),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(!certificates.is_empty(), "SSL_CERT_FILE {} contains no certificates", path.display());
    Some(
        TlsConfig::builder()
            .provider(TLS_PROVIDER)
            .root_certs(RootCerts::from(certificates))
            .build(),
    )
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
