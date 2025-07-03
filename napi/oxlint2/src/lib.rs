use std::process::{ExitCode, Termination};

use napi_derive::napi;

use oxlint::lint as oxlint_lint;

#[expect(clippy::allow_attributes)]
#[allow(clippy::trailing_empty_array, clippy::unused_async)] // https://github.com/napi-rs/napi-rs/issues/2758
#[napi]
pub async fn lint() -> bool {
    oxlint_lint().report() == ExitCode::SUCCESS
}
