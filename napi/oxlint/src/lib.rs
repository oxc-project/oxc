use oxlint::lint as oxlint_lint;

use napi_derive::napi;
use std::process::{ExitCode, Termination};

#[napi]
pub fn lint() -> bool {
    oxlint_lint().report() == ExitCode::SUCCESS
}
