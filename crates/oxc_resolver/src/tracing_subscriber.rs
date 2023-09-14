use once_cell::sync::OnceCell;
use tracing_subscriber::{fmt, prelude::*, registry, EnvFilter};

static SUBSCRIBER: OnceCell<()> = OnceCell::new();

pub fn init() {
    SUBSCRIBER.get_or_init(|| {
        registry().with(fmt::layer()).with(EnvFilter::from_env("OXC_RESOLVER")).init();
    });
}
