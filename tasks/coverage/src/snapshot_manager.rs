use std::path::Path;

use oxc_tasks_common::Snapshot;

use crate::{
    AppArgs,
    coverage::CoverageReport,
    snap_root,
    suite::{Case, TestResult},
    workspace_root,
};

/// Handles snapshot creation and management
pub struct SnapshotManager;

impl SnapshotManager {
    /// Create and save snapshots for test errors
    /// # Errors
    pub fn snapshot_errors<T: Case>(
        name: &str,
        test_root: &Path,
        test_cases: &[T],
        report: &CoverageReport<T>,
    ) -> std::io::Result<()> {
        let snapshot_path = workspace_root().join(test_root);
        let show_commit = !snapshot_path.to_string_lossy().contains("misc");
        let snapshot = Snapshot::new(&snapshot_path, show_commit);

        let mut tests = test_cases
            .iter()
            .filter(|case| matches!(case.test_result(), TestResult::CorrectError(_, _)))
            .collect::<Vec<_>>();

        tests.sort_by_key(|case| case.path());

        let mut out: Vec<u8> = vec![];

        let args = AppArgs { detail: true, ..AppArgs::default() };
        report.print(name, &args, &mut out)?;

        for case in &tests {
            if let TestResult::CorrectError(error, _) = &case.test_result() {
                out.extend(error.as_bytes());
            }
        }

        let path = snap_root().join(format!("{}.snap", name.to_lowercase()));
        let out = String::from_utf8(out).unwrap();
        snapshot.save(&path, &out);
        Ok(())
    }
}
