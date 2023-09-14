use once_cell::sync::OnceCell;
use std::str::FromStr;
use tracing_subscriber::{filter::Targets, fmt, prelude::*, registry};

static SUBSCRIBER: OnceCell<()> = OnceCell::new();

pub fn init() {
    SUBSCRIBER.get_or_init(|| {
        registry()
            .with(
                std::env::var("OXC_RESOLVER").map_or_else(
                    |_| Targets::new(),
                    |env_var| Targets::from_str(&env_var).unwrap(),
                ),
            )
            .with(fmt::layer())
            .init();
    });
}
