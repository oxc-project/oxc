use std::path::{Path, PathBuf};

use crate::{ResolveError, ResolveOptions, Resolver};

#[test]
fn builtins_off() {
    let f = Path::new("/");
    let resolver = Resolver::default();
    let resolved_path = resolver.resolve(f, "zlib").map(|r| r.full_path());
    assert_eq!(resolved_path, Err(ResolveError::NotFound(PathBuf::from("/"))));
}

#[test]
fn builtins() {
    let f = Path::new("/");

    let resolver =
        Resolver::new(ResolveOptions { builtin_modules: true, ..ResolveOptions::default() });

    let pass = [
        "_http_agent",
        "_http_client",
        "_http_common",
        "_http_incoming",
        "_http_outgoing",
        "_http_server",
        "_stream_duplex",
        "_stream_passthrough",
        "_stream_readable",
        "_stream_transform",
        "_stream_wrap",
        "_stream_writable",
        "_tls_common",
        "_tls_wrap",
        "assert",
        "assert/strict",
        "async_hooks",
        "buffer",
        "child_process",
        "cluster",
        "console",
        "constants",
        "crypto",
        "dgram",
        "diagnostics_channel",
        "dns",
        "dns/promises",
        "domain",
        "events",
        "fs",
        "fs/promises",
        "http",
        "http2",
        "https",
        "inspector",
        "module",
        "net",
        "os",
        "path",
        "path/posix",
        "path/win32",
        "perf_hooks",
        "process",
        "punycode",
        "querystring",
        "readline",
        "repl",
        "stream",
        "stream/consumers",
        "stream/promises",
        "stream/web",
        "string_decoder",
        "sys",
        "timers",
        "timers/promises",
        "tls",
        "trace_events",
        "tty",
        "url",
        "util",
        "util/types",
        "v8",
        "vm",
        "worker_threads",
        "zlib",
    ];

    for request in pass {
        let resolved_path = resolver.resolve(f, request).map(|r| r.full_path());
        assert_eq!(resolved_path, Err(ResolveError::Builtin(request.to_string())), "{request}");
    }

    for request in pass {
        let request = format!("node:{request}");
        let resolved_path = resolver.resolve(f, &request).map(|r| r.full_path());
        assert_eq!(resolved_path, Err(ResolveError::Builtin(request.to_string())), "{request}");
    }
}
