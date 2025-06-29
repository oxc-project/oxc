use std::process::{ExitCode, Termination};

use napi_derive::napi;

use oxlint::lint as oxlint_lint;

#[napi]
pub fn lint() -> bool {
    oxlint_lint().report() == ExitCode::SUCCESS
}
